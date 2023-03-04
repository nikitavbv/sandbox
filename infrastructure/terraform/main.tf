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
}

resource vultr_vpc backend {
  region = data.vultr_region.waw.id
  description = "sandbox-backend"
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

resource vultr_instance frontend {
  plan = data.vultr_plan.small_cpu_instance.id
  region = data.vultr_region.waw.id
  os_id = data.vultr_os.arch_linux.id
  label = "sandbox-frontend"
  tags = var.tags
  hostname = "sandbox-frontend"
  enable_ipv6 = true
  vpc_ids = [vultr_vpc.frontend.id]
  user_data = <<SCRIPT
#!/usr/bin/env bash
echo "does it work?" > /root/test
SCRIPT

  lifecycle {
    ignore_changes = [server_status] 
  }
}

resource cloudflare_record frontend_1 {
  zone_id = file(".secrets/cloudflare_zone_id")
  type = "A"
  name = "sandbox-frontend-1"
  value = vultr_instance.frontend.internal_ip
  proxied = false
  allow_overwrite = true
  comment = "sandbox frontend instance internal"
  ttl = 300
}