terraform {
  required_version = ">= 1.10.0, < 2.0.0"

  required_providers {
    contabo = {
      source  = "contabo/contabo"
      version = "= 0.1.44"
    }
  }

  backend "s3" {}
}

provider "contabo" {
  # Credentials are read from the provider-supported environment variables:
  # CNTB_OAUTH2_CLIENT_ID, CNTB_OAUTH2_CLIENT_SECRET,
  # CNTB_OAUTH2_USER, CNTB_OAUTH2_PASS.
}

variable "manage_instance" {
  type    = bool
  default = false
}

variable "manage_firewall" {
  type    = bool
  default = false
}

variable "instance_display_name" {
  type    = string
  default = "dubbridge-poc-v1"
}

variable "product_id" {
  type      = string
  default   = null
  nullable  = true
  description = "Exact Contabo product/SKU. Frozen only at T6d.C2 after checkout verification."
}

variable "image_id" {
  type      = string
  default   = null
  nullable  = true
  description = "Exact Contabo Linux image id. Must be explicitly selected before creation."
}

variable "region" {
  type    = string
  default = "EU"
}

variable "contract_period_months" {
  type    = number
  default = 1
}

variable "ssh_secret_ids" {
  type        = list(number)
  default     = []
  description = "Existing Contabo Secret Management SSH public-key IDs."
}

variable "operator_ssh_ipv4_cidrs" {
  type        = list(string)
  default     = []
  description = "Operator IPv4 CIDRs allowed to SSH. Empty means no SSH firewall rule."
}

variable "operator_ssh_ipv6_cidrs" {
  type        = list(string)
  default     = []
  description = "Operator IPv6 CIDRs allowed to SSH. Empty means no SSH firewall rule."
}

locals {
  public_ipv4 = ["0.0.0.0/0"]
  public_ipv6 = ["::/0"]
}

check "instance_create_inputs" {
  assert {
    condition = !var.manage_instance || (
      var.product_id != null &&
      trimspace(var.product_id) != "" &&
      var.image_id != null &&
      trimspace(var.image_id) != "" &&
      length(var.ssh_secret_ids) > 0
    )
    error_message = "Instance creation requires explicit product_id, image_id and at least one SSH secret id."
  }
}

check "firewall_requires_instance" {
  assert {
    condition     = !var.manage_firewall || var.manage_instance
    error_message = "Firewall management requires manage_instance=true so attachment is unambiguous."
  }
}

resource "contabo_instance" "app" {
  count = var.manage_instance ? 1 : 0

  display_name = var.instance_display_name
  product_id   = var.product_id
  image_id     = var.image_id
  region       = var.region
  period       = var.contract_period_months
  default_user = "root"
  ssh_keys     = var.ssh_secret_ids

  lifecycle {
    prevent_destroy = true
  }
}

resource "contabo_firewall" "app" {
  count = var.manage_firewall ? 1 : 0

  name         = "dubbridge-poc-v1"
  description  = "DubBridge POC public edge: SSH restricted, HTTP/HTTPS public"
  status       = "active"
  instance_ids = [tonumber(contabo_instance.app[0].id)]

  rules {
    dynamic "inbound" {
      for_each = (length(var.operator_ssh_ipv4_cidrs) + length(var.operator_ssh_ipv6_cidrs)) > 0 ? [1] : []
      content {
        action     = "accept"
        protocol   = "tcp"
        dest_ports = ["22"]
        status     = "active"
        src_cidr {
          ipv4 = var.operator_ssh_ipv4_cidrs
          ipv6 = var.operator_ssh_ipv6_cidrs
        }
      }
    }

    inbound {
      action     = "accept"
      protocol   = "tcp"
      dest_ports = ["80"]
      status     = "active"
      src_cidr {
        ipv4 = local.public_ipv4
        ipv6 = local.public_ipv6
      }
    }

    inbound {
      action     = "accept"
      protocol   = "tcp"
      dest_ports = ["443"]
      status     = "active"
      src_cidr {
        ipv4 = local.public_ipv4
        ipv6 = local.public_ipv6
      }
    }
  }

  lifecycle {
    prevent_destroy = true
  }
}

output "instance_id" {
  value = var.manage_instance ? contabo_instance.app[0].id : null
}

output "instance_ip_config" {
  value = var.manage_instance ? contabo_instance.app[0].ip_config : null
}

output "firewall_id" {
  value = var.manage_firewall ? contabo_firewall.app[0].id : null
}
