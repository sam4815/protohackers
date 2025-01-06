project := `gcloud config get-value project`

[working-directory: 'terraform']
init:
  terraform init
  terraform apply -auto-approve

[working-directory: 'terraform']
destroy:
  terraform destroy -auto-approve

ssh:
  gcloud compute ssh protohackers

deploy directory:
  podman build --build-arg SOURCE_DIR={{directory}} --platform linux/amd64 -t gcr.io/{{project}}/{{directory}} .
  podman push gcr.io/{{project}}/{{directory}}
  gcloud compute ssh protohackers --command "sudo systemctl start konlet-startup"
