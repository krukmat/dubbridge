# S-230-T6d.O1.A/B — OCI Always Free infrastructure descriptor.
# DELIBERATELY NO APPLY: account-specific limits, region, state and consent
# are not yet proven. All resources are disabled by default.
terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "= 9.9.0"
    }
  }
  # Remote state must be bootstrapped and proven before O2;
  # NEVER apply while init -backend=false or using local state.
  backend "s3" {}
}

provider "oci" {
  # OCI_CONFIG_FILE / OCI_CONFIG_PROFILE or other supported provider
  # environment authentication; no private key material in the repo.
  region = var.region
}

variable "enable_resources" {
  type        = bool
  default     = false
  description = "Default=false. Changing to true alone is NOT authorization to apply."
}

variable "region" {
  type        = string
  default     = null
  description = "Verified OCI home-region identifier (O0.B)."
}

variable "compartment_id" {
  type        = string
  default     = ""
  description = "Explicit OCI compartment OCID."
}

variable "availability_domain" {
  type        = string
  default     = ""
  description = "Verified AD name supporting Always Free A1 capacity."
}

variable "arm64_image_ocid" {
  type        = string
  default     = ""
  description = "Explicit compatible ARM64 Ubuntu or Oracle Linux image OCID."
}

variable "ssh_public_key" {
  type        = string
  default     = ""
  description = "Only public key text. NEVER put a private key in tfvars."
}

variable "operator_ssh_ipv4_cidr" {
  type        = string
  default     = ""
  description = "Operator /32 address (no 0.0.0.0/0 SSH)."
  validation {
    condition = var.operator_ssh_ipv4_cidr == "" || (
      can(cidrhost(var.operator_ssh_ipv4_cidr, 0)) &&
      endswith(var.operator_ssh_ipv4_cidr, "/32") &&
      var.operator_ssh_ipv4_cidr != "0.0.0.0/0"
    )
    error_message = "SSH requires a single-operator IPv4 /32."
  }
}

variable "ocpus" {
  type    = number
  default = 2
  validation {
    condition     = var.ocpus == 2
    error_message = "Always Free tenancy planning baseline is exactly 2 OCPU."
  }
}
variable "memory_gb" {
  type    = number
  default = 12
  validation {
    condition     = var.memory_gb == 12
    error_message = "Always Free tenancy planning baseline is exactly 12 GB."
  }
}
variable "boot_gb" {
  type    = number
  default = 50
  validation {
    condition     = var.boot_gb == 50
    error_message = "The O0 cost envelope reserves only 50 GB for boot."
  }
}

check "required_account_inputs" {
  assert {
    condition = !var.enable_resources || (
      length(var.compartment_id) > 10 &&
      try(length(var.region), 0) > 2 &&
      length(var.availability_domain) > 2 &&
      length(var.arm64_image_ocid) > 10 &&
      length(var.ssh_public_key) > 40 &&
      endswith(var.operator_ssh_ipv4_cidr, "/32")
    )
    error_message = "O2 requires region, compartment, AD, arm64 image, SSH public key and restricted operator CIDR."
  }
}

locals {
  deploy = var.enable_resources ? 1 : 0
  tags   = { project = "dubbridge", stage = "poc", budget = "always-free-only" }
}

resource "oci_core_vcn" "poc" {
  count          = local.deploy
  compartment_id = var.compartment_id
  cidr_blocks    = ["10.76.0.0/16"]
  display_name   = "dubbridge-poc-vcn"
  dns_label      = "dubbridgepoc"
  freeform_tags  = local.tags
  lifecycle { prevent_destroy = true }
}

resource "oci_core_internet_gateway" "poc" {
  count          = local.deploy
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.poc[0].id
  display_name   = "dubbridge-poc-igw"
  enabled        = true
  lifecycle { prevent_destroy = true }
}

resource "oci_core_route_table" "poc" {
  count          = local.deploy
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.poc[0].id
  display_name   = "dubbridge-poc-route"
  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.poc[0].id
  }
  lifecycle { prevent_destroy = true }
}

# Explicit security list: do not inherit the VCN's default SSH-from-world rule.
resource "oci_core_security_list" "poc" {
  count          = local.deploy
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.poc[0].id
  display_name   = "dubbridge-poc-firewall"

  egress_security_rules {
    protocol    = "all"
    destination = "0.0.0.0/0"
  }
  ingress_security_rules {
    protocol = "6"
    source   = var.operator_ssh_ipv4_cidr
    tcp_options {
      min = 22
      max = 22
    }
  }
  ingress_security_rules {
    protocol = "6"
    source   = "0.0.0.0/0"
    tcp_options {
      min = 80
      max = 80
    }
  }
  ingress_security_rules {
    protocol = "6"
    source   = "0.0.0.0/0"
    tcp_options {
      min = 443
      max = 443
    }
  }
  lifecycle { prevent_destroy = true }
}

resource "oci_core_subnet" "poc" {
  count                      = local.deploy
  compartment_id             = var.compartment_id
  vcn_id                     = oci_core_vcn.poc[0].id
  cidr_block                 = "10.76.10.0/24"
  display_name               = "dubbridge-poc-public-subnet"
  dns_label                  = "poc"
  route_table_id             = oci_core_route_table.poc[0].id
  security_list_ids          = [oci_core_security_list.poc[0].id]
  prohibit_public_ip_on_vnic = false
  lifecycle { prevent_destroy = true }
}

resource "oci_core_instance" "poc" {
  count               = local.deploy
  compartment_id      = var.compartment_id
  availability_domain = var.availability_domain
  display_name        = "dubbridge-poc-a1"
  shape               = "VM.Standard.A1.Flex"
  shape_config {
    ocpus         = var.ocpus
    memory_in_gbs = var.memory_gb
  }
  create_vnic_details {
    subnet_id        = oci_core_subnet.poc[0].id
    assign_public_ip = true
    hostname_label   = "dubbridge"
  }
  source_details {
    source_type            = "image"
    source_id              = var.arm64_image_ocid
    boot_volume_size_in_gbs = var.boot_gb
  }
  metadata = {
    ssh_authorized_keys = var.ssh_public_key
  }
  freeform_tags = local.tags
  lifecycle { prevent_destroy = true }
}

output "poc_instance_id" {
  value = var.enable_resources ? oci_core_instance.poc[0].id : null
}
output "poc_public_ip" {
  value = var.enable_resources ? oci_core_instance.poc[0].public_ip : null
}
