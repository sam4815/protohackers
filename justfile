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

logs:
  @gcloud compute ssh protohackers -- 'docker logs $(docker ps -q) --timestamps'

deploy directory $IMAGE=("gcr.io/" + project + "/" + directory):
  podman build --build-arg SOURCE_DIR={{directory}} --platform linux/amd64 -t $IMAGE .
  podman push $IMAGE
  gcloud compute instances add-metadata protohackers \
    --metadata=gce-container-declaration="$(envsubst < container-spec.yaml)"
  gcloud compute ssh protohackers --command "sudo systemctl start konlet-startup"
