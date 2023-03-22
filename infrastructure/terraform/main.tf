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
  
  lifecycle {
    prevent_destroy = true
  }
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
  firewall_group_id = vultr_firewall_group.frontend.id

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
  firewall_group_id = vultr_firewall_group.envoy.id

  lifecycle {
    ignore_changes = [server_status]
  }
}

resource cloudflare_record envoy_1 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-envoy-1"
  value = vultr_instance.envoy_1.main_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox envoy-1 instance"
  ttl = 300
}

resource vultr_instance envoy_2 {
  plan = data.vultr_plan.small_cpu_instance.id
  region = data.vultr_region.waw.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-envoy-2"
  tags = var.tags
  hostname = "sandbox-envoy-2"
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
  firewall_group_id = vultr_firewall_group.envoy.id

  lifecycle {
    ignore_changes = [server_status]
  }
}

resource cloudflare_record envoy_2 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-envoy-2"
  value = vultr_instance.envoy_2.main_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox envoy-2 instance"
  ttl = 300
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
systemctl enable docker
export OBJECT_STORAGE_ACCESS_KEY="${file(".secrets/object_storage_access_key")}"
export OBJECT_STORAGE_SECRET_KEY="${file(".secrets/object_storage_secret_key")}"
export HOSTNAME=$(cat /etc/hostname)
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/sandbox.toml | envsubst > /root/config.toml
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/sandbox.service | envsubst > /etc/systemd/system/sandbox.service
systemctl enable sandbox
ufw allow 8080
reboot
SCRIPT
  firewall_group_id = vultr_firewall_group.cpu_workers.id

  lifecycle {
    ignore_changes = [server_status]
  }
}

resource cloudflare_record cpu_1 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-cpu-1"
  value = vultr_instance.cpu_1.internal_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox cpu worker internal"
  ttl = 300
}

// cloud instance for gpu worker
data vultr_region fra {
  filter {
    name = "id"
    values = ["fra"]
  }
}

data vultr_plan gpu_a100_10vram_instance {
  filter {
    name = "locations"
    values = [data.vultr_region.fra.id]
  }

  filter {
    name = "id"
    values = ["vcg-a100-2c-15g-10vram"]
  }
}

/*resource vultr_instance gpu_1 {
  plan = data.vultr_plan.gpu_a100_10vram_instance.id
  region = data.vultr_region.fra.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-gpu-1"
  tags = var.tags
  hostname = "sandbox-gpu-1"
  enable_ipv6 = true
  vpc_ids = []
  user_data = <<SCRIPT
#!/usr/bin/env bash
pacman -S --noconfirm gettext protobuf
export OBJECT_STORAGE_ACCESS_KEY="${file(".secrets/object_storage_access_key")}"
export OBJECT_STORAGE_SECRET_KEY="${file(".secrets/object_storage_secret_key")}"
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/sandbox.toml | envsubst > /root/config.toml
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/sandbox-gpu.sh > /root/sandbox-gpu.sh
chmod +x /root/sandbox-gpu.sh
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/sandbox-gpu.service > /etc/systemd/system/sandbox.service
systemctl enable sandbox
ufw allow 8080
SCRIPT
  firewall_group_id = vultr_firewall_group.gpu_workers.id

  lifecycle {
    ignore_changes = [server_status]
  }
}

resource cloudflare_record gpu_1 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-gpu-1"
  value = vultr_instance.gpu_1.main_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox gpu-1 worker internal"
  ttl = 300
}

resource vultr_instance gpu_2 {
  plan = data.vultr_plan.gpu_a100_10vram_instance.id
  region = data.vultr_region.fra.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-gpu-2"
  tags = var.tags
  hostname = "sandbox-gpu-2"
  enable_ipv6 = true
  vpc_ids = []
  user_data = <<SCRIPT
#!/usr/bin/env bash
pacman -S --noconfirm gettext protobuf
export OBJECT_STORAGE_ACCESS_KEY="${file(".secrets/object_storage_access_key")}"
export OBJECT_STORAGE_SECRET_KEY="${file(".secrets/object_storage_secret_key")}"
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/sandbox.toml | envsubst > /root/config.toml
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/sandbox-gpu.sh > /root/sandbox-gpu.sh
chmod +x /root/sandbox-gpu.sh
curl https://raw.githubusercontent.com/nikitavbv/sandbox/master/infrastructure/systemd/sandbox-gpu.service > /etc/systemd/system/sandbox.service
systemctl enable sandbox
ufw allow 8080
SCRIPT
  firewall_group_id = vultr_firewall_group.gpu_workers.id

  lifecycle {
    ignore_changes = [server_status]
  }
}

resource cloudflare_record gpu_2 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-gpu-2"
  value = vultr_instance.gpu_2.main_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox gpu-2 worker internal"
  ttl = 300
}*/

resource cloudflare_load_balancer sandbox_lb {
  zone_id = file(".secrets/cloudflare_zone_id")
  name = "sandbox.nikitavbv.com"
  default_pool_ids = [cloudflare_load_balancer_pool.sandbox_main_pool.id]
  fallback_pool_id = cloudflare_load_balancer_pool.sandbox_main_pool.id
  description = "load balancer for sandbox"
  proxied = true
}

resource cloudflare_load_balancer_pool sandbox_main_pool {
  account_id = file(".secrets/cloudflare_account_id")
  name = "sandbox-main"
  
  origins {
    name = "envoy-1"
    address = "sandbox-envoy-1.nikitavbv.com"
  }

  origins {
    name = "envoy-2"
    address = "sandbox-envoy-2.nikitavbv.com"
  }

  monitor = cloudflare_load_balancer_monitor.healthcheck.id
}

resource cloudflare_load_balancer_monitor healthcheck {
  account_id = file(".secrets/cloudflare_account_id")
  description = "healthcheck"
  type = "https"
  expected_codes = "2xx"
  method = "GET"
  path = "/healthz"
  interval = 60
  retries = 5
}

resource cloudflare_notification_policy origin_status {
  account_id = file(".secrets/cloudflare_account_id")
  name = "Notify when origin status changes"
  enabled = true
  alert_type = "load_balancing_health_alert"

  email_integration {
    id = "nikitavbv@gmail.com"
  }

  filters {
    pool_id = [cloudflare_load_balancer_pool.sandbox_main_pool.id]
  }
}

// firewall
resource vultr_firewall_group envoy {
  description = "envoy"
}

resource vultr_firewall_rule allow_https_from_cloudflare_to_envoy {
  firewall_group_id = vultr_firewall_group.envoy.id
  protocol = "tcp"
  ip_type = "v4"
  source = "cloudflare"
  subnet = "0.0.0.0"
  subnet_size = 0
  port = "443"
}

resource vultr_firewall_group gpu_workers {
  description = "sandbox-gpu"
}

resource vultr_firewall_rule allow_https_from_envoy1_to_gpu {
  firewall_group_id = vultr_firewall_group.gpu_workers.id
  protocol = "tcp"
  ip_type = "v4"
  subnet = vultr_instance.envoy_1.main_ip
  subnet_size = 32
  port = "8080"
}

resource vultr_firewall_rule allow_https_from_envoy2_to_gpu {
  firewall_group_id = vultr_firewall_group.gpu_workers.id
  protocol = "tcp"
  ip_type = "v4"
  subnet = vultr_instance.envoy_2.main_ip
  subnet_size = 32
  port = "8080"
}

resource vultr_firewall_group cpu_workers {
  description = "sandbox-cpu"
}

resource vultr_firewall_rule allow_https_from_backend_vpc_to_cpu {
  firewall_group_id = vultr_firewall_group.cpu_workers.id
  protocol = "tcp"
  ip_type = "v4"
  subnet = vultr_vpc.backend.v4_subnet
  subnet_size = vultr_vpc.backend.v4_subnet_mask
  port = "8080"
}

resource vultr_firewall_group frontend {
  description = "sandbox-frontend"
}

resource vultr_firewall_rule allow_https_from_frontend_vpc_to_frontend {
  firewall_group_id = vultr_firewall_group.frontend.id
  protocol = "tcp"
  ip_type = "v4"
  subnet = vultr_vpc.frontend.v4_subnet
  subnet_size = vultr_vpc.frontend.v4_subnet_mask
  port = "8080"
}
