resource "aws_security_group" "nodes" {
  name        = "${var.cluster_name}_sg"
  vpc_id      = aws_vpc.vpc.id
}

resource "aws_security_group_rule" "outbound_traffic" {
  security_group_id = aws_security_group.nodes.id

  description      = "Allow all outbound traffic"
  type             = "egress"
  from_port        = 0
  to_port          = 0
  protocol         = "-1"
  cidr_blocks      = ["0.0.0.0/0"]
  ipv6_cidr_blocks = ["::/0"]
}

resource "aws_security_group_rule" "ssh" {
  security_group_id = aws_security_group.nodes.id

  description = "Allow SSH access"
  type        = "ingress"
  from_port   = 22
  to_port     = 22
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "rpc" {
  security_group_id = aws_security_group.nodes.id

  description = "Allow RPC access"
  type        = "ingress"
  from_port   = 9944
  to_port     = 9944
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "prometheus" {
  security_group_id = aws_security_group.nodes.id

  description = "Allow Prometheus access"
  type        = "ingress"
  from_port   = 9615
  to_port     = 9615
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "p2p" {
  security_group_id = aws_security_group.nodes.id

  description = "Allow RPC access"
  type        = "ingress"
  from_port   = 30333
  to_port     = 30333
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "internal_traffic" {
  security_group_id = aws_security_group.nodes.id

  description = "Allow all internal traffic"
  type        = "ingress"
  from_port   = 0
  to_port     = 0
  protocol    = "-1"
  source_security_group_id = aws_security_group.nodes.id
}
