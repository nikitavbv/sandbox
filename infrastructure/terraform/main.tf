terraform {
  required_providers {
    vultr = {
      source = "vultr/vultr"
      version = "2.12.1"
    }
  }
}

provider vultr {
  api_key = "${file(".secrets/vultr_api_key")}"
  rate_limit = 100
  retry_limit = 3
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