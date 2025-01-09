provider "google" {
  credentials = file("terraform-key.json")
  project     = "proto-446020"
  region      = "us-central1"
}

resource "google_compute_firewall" "default" {
  name    = "allow-http"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["8080"]
  }

  source_ranges = ["0.0.0.0/0"]
}

resource "google_compute_instance" "protohackers" {
  name         = "protohackers"
  machine_type = "e2-micro"
  zone         = "us-central1-a"

  boot_disk {
    initialize_params {
      image = "projects/cos-cloud/global/images/family/cos-stable"
    }
  }

  network_interface {
    network = "default"

    access_config {
      # Required to assign a public IP
    }
  }

  service_account {
    email  = "default"
    scopes = ["cloud-platform"]
  }

  tags = ["http-server"]
}

output "external_ip" {
  value = google_compute_instance.protohackers.network_interface[0].access_config[0].nat_ip
  description = "The external IP address of the VM"
}
