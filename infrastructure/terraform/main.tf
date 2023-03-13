terraform {
  required_providers {
    vultr = {
      source = "vultr/vultr"
      version = "2.12.1"
    }
    
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 3.0"
    }
  }
}

provider vultr {
  api_key = file(".secrets/vultr_api_key")
  rate_limit = 100
  retry_limit = 3
}

provider cloudflare {
  api_token = file(".secrets/cloudflare_api_token")
}

variable tags {
  default = ["sandbox"]
}

data vultr_object_storage_cluster object_storage_ams {
  filter {
    name = "region"
    values = ["ams"]
  }
}

resource vultr_object_storage object_storage {
  cluster_id = data.vultr_object_storage_cluster.object_storage_ams.id
  label = "${file(".secrets/object_storage_cluster_name")}"
}

// VPCs
data vultr_region waw {
  filter {
    name = "id"
    values = ["waw"]
  }
}

resource vultr_vpc frontend {
  region = data.vultr_region.waw.id
  description = "sandbox-frontend"
  v4_subnet = "10.27.96.0"
  v4_subnet_mask = 20
}

resource vultr_vpc backend {
  region = data.vultr_region.waw.id
  description = "sandbox-backend"
  v4_subnet = "10.27.128.0"
  v4_subnet_mask = 20
}

// cloud instance to server frontend files
data vultr_plan small_cpu_instance {
  filter {
    name = "locations"
    values = [data.vultr_region.waw.id]
  }

  filter {
    name = "id"
    values = ["vc2-1c-1gb"]
  }
}

data vultr_os arch_linux {
  filter {
    name = "family"
    values = ["archlinux"]
  }
}

resource vultr_instance frontend_1 {
  plan = data.vultr_plan.small_cpu_instance.id
  region = data.vultr_region.waw.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-frontend-1"
  tags = var.tags
  hostname = "sandbox-frontend-1"
  enable_ipv6 = true
  vpc_ids = [vultr_vpc.frontend.id]
  user_data = <<SCRIPT
#!/usr/bin/env bash
pacman -S --noconfirm bridge-utils gettext
pacman -S --noconfirm docker
systemctl enable docker
export OBJECT_STORAGE_ACCESS_KEY="${file(".secrets/object_storage_access_key")}"
export OBJECT_STORAGE_SECRET_KEY="${file(".secrets/object_storage_secret_key")}"
mkdir /root/conf
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/s3-proxy.yaml | envsubst > /root/conf/config.yaml
wget https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/s3-proxy.service
mv s3-proxy.service /etc/systemd/system/
systemctl enable s3-proxy
ufw allow 8080
reboot
SCRIPT

  lifecycle {
    ignore_changes = [server_status] 
  }
}

resource cloudflare_record frontend_1 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-frontend-1"
  value = vultr_instance.frontend_1.internal_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox frontend instance internal"
  ttl = 300
}

// cloud instance for envoy proxy
resource vultr_instance envoy_1 {
  plan = data.vultr_plan.small_cpu_instance.id
  region = data.vultr_region.waw.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-envoy-1"
  tags = var.tags
  hostname = "sandbox-envoy-1"
  enable_ipv6 = true
  vpc_ids = [vultr_vpc.frontend.id, vultr_vpc.backend.id]
  user_data = <<SCRIPT
#!/usr/bin/env bash
pacman -S --noconfirm bridge-utils gettext docker
export SSL_CERTIFICATE=$(echo "${base64encode(file(".secrets/ssl_certificate_envoy"))}" | base64 -d)
export SSL_PRIVATE_KEY=$(echo "${base64encode(file(".secrets/ssl_private_key_envoy"))}" | base64 -d)
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/envoy.yaml | envsubst > /root/config.yaml
wget https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/envoy.service
mv envoy.service /etc/systemd/system/
systemctl enable envoy
ufw allow 80
reboot
SCRIPT

  lifecycle {
    ignore_changes = [server_status]
  }
}

// cloud instance for cpu worker
data vultr_plan high_performance_amd_4c_instance {
  filter {
    name = "locations"
    values = [data.vultr_region.waw.id]
  }

  filter {
    name = "id"
    values = ["vhp-4c-8gb-amd"]
  }
}

resource vultr_instance cpu_1 {
  plan = data.vultr_plan.high_performance_amd_4c_instance.id
  region = data.vultr_region.waw.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-cpu-1"
  tags = var.tags
  hostname = "sandbox-cpu-1"
  enable_ipv6 = true
  vpc_ids = [vultr_vpc.backend.id]
  user_data = <<SCRIPT
#!/usr/bin/env bash
pacman -S --noconfirm bridge-utils gettext docker

systemctl enable envoy
ufw allow 8080
SCRIPT

  lifecycle {
    ignore_changes = [server_status]
  }
}