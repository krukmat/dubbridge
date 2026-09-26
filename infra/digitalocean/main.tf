terraform {
  required_version = ">= 1.10.0, < 2.0.0"

  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }

  backend "s3" {}
}

variable "digitalocean_token" {
  type      = string
  sensitive = true
  default   = null
}

variable "spaces_access_key_id" {
  type      = string
  sensitive = true
  default   = null
}

variable "spaces_secret_access_key" {
  type      = string
  sensitive = true
  default   = null
}

variable "region" {
  type    = string
  default = "ams3"
}

variable "domain" {
  type    = string
  default = "iotforce.es"
}

variable "hostname" {
  type    = string
  default = "poc"
}

variable "droplet_name" {
  type    = string
  default = "dubbridge-poc-v1"
}

variable "droplet_size" {
  type    = string
  default = "s-2vcpu-4gb"
}

variable "droplet_image" {
  type    = string
  default = "ubuntu-24-04-x64"
}

variable "ssh_key_ids" {
  type    = list(number)
  default = []
}

variable "operator_ssh_cidrs" {
  type    = list(string)
  default = []
}

variable "database_name" {
  type    = string
  default = "dubbridge-poc-v1"
}

variable "database_version" {
  type    = string
  default = "17"
}

variable "database_size" {
  type    = string
  default = "db-s-1vcpu-1gb"
}

variable "media_space_name" {
  type    = string
  default = "dubbridge-poc-v1"
}

variable "manage_droplet" {
  type    = bool
  default = false
}

variable "manage_firewall" {
  type    = bool
  default = false
}

variable "manage_database" {
  type    = bool
  default = false
}

variable "manage_media_space" {
  type    = bool
  default = false
}

variable "manage_dns" {
  type    = bool
  default = false
}

provider "digitalocean" {
  token             = var.digitalocean_token
  spaces_access_id  = var.spaces_access_key_id
  spaces_secret_key = var.spaces_secret_access_key
}

locals {
  public_cidrs = ["0.0.0.0/0", "::/0"]
}

resource "digitalocean_droplet" "app" {
  count = var.manage_droplet ? 1 : 0

  name       = var.droplet_name
  region     = var.region
  size       = var.droplet_size
  image      = var.droplet_image
  ssh_keys   = var.ssh_key_ids
  monitoring = true
  ipv6       = true
  tags       = ["dubbridge", "poc-v1", "production"]

  lifecycle {
    prevent_destroy = true
  }
}

resource "digitalocean_firewall" "app" {
  count = var.manage_firewall ? 1 : 0

  name        = "dubbridge-poc-v1"
  droplet_ids = [digitalocean_droplet.app[0].id]

  dynamic "inbound_rule" {
    for_each = length(var.operator_ssh_cidrs) > 0 ? [1] : []
    content {
      protocol         = "tcp"
      port_range       = "22"
      source_addresses = var.operator_ssh_cidrs
    }
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "80"
    source_addresses = local.public_cidrs
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "443"
    source_addresses = local.public_cidrs
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = local.public_cidrs
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = local.public_cidrs
  }

  outbound_rule {
    protocol              = "icmp"
    destination_addresses = local.public_cidrs
  }

  lifecycle {
    prevent_destroy = true
  }
}

resource "digitalocean_database_cluster" "postgres" {
  count = var.manage_database ? 1 : 0

  name       = var.database_name
  engine     = "pg"
  version    = var.database_version
  size       = var.database_size
  region     = var.region
  node_count = 1

  lifecycle {
    prevent_destroy = true
  }
}

resource "digitalocean_spaces_bucket" "media" {
  count = var.manage_media_space ? 1 : 0

  name   = var.media_space_name
  region = var.region
  acl    = "private"

  lifecycle {
    prevent_destroy = true
  }
}

resource "digitalocean_record" "poc" {
  count = var.manage_dns ? 1 : 0

  domain = var.domain
  type   = "A"
  name   = var.hostname
  value  = digitalocean_droplet.app[0].ipv4_address
  ttl    = 300

  lifecycle {
    prevent_destroy = true
  }
}
