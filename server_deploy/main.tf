provider "google" {
    project = "automation-nk"
    region  = "us-central1"
    zone    = "us-central1-c"
}

# Create VPC
resource "google_compute_network" "vpc" {
    name                    = "${var.app_name}-net"
    auto_create_subnetworks = "true"
}

# Create VM (with VPC)
resource "google_compute_instance" "vm" {
    name         = var.app_name
    machine_type = "e2-micro"

    boot_disk {
        initialize_params {
            image = "ubuntu-2004-focal-v20211118"
        }
    }

    network_interface {
        network = google_compute_network.vpc.self_link
        access_config {
            nat_ip = var.static_ip
        }
    }

    tags = [ "udp" ]

    allow_stopping_for_update = false
    metadata_startup_script = join("\n", ["echo \"${var.git_credentials}\" > ~/.git-credentials;", file("${path.module}/setup.bash")])
}

# Allow UDP to server
resource "google_compute_firewall" "udp" {
    name = "${var.app_name}-allow-udp"
    network = google_compute_network.vpc.name
    source_ranges = [ "0.0.0.0/0" ]
    target_tags = [ "udp" ]
    allow {
        protocol = "udp"
        ports    = [ "8888" ]
    }
}

# Allow SSH
resource "google_compute_firewall" "ssh" {
    name = "${var.app_name}-allow-ssh"
    network = google_compute_network.vpc.name
    source_ranges = [ "0.0.0.0/0" ]
    allow {
        protocol = "tcp"
        ports = [ "22" ]
    }
}
