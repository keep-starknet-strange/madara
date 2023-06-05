resource "tls_private_key" "priv_key" {
  algorithm = "RSA"
  rsa_bits  = 4096
}

resource "aws_key_pair" "key_pair" {
  key_name = var.node_ssh_key_name
  public_key = tls_private_key.priv_key.public_key_openssh
}

resource "local_file" "cloud_pem" {
  filename = "${path.module}/../../ansible/madara.pem"
  content = tls_private_key.priv_key.private_key_pem
}
