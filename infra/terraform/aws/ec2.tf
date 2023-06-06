data "aws_ami" "ec2_instance" {
  most_recent = true
  owners = ["136693071363"]

  filter {
    name = "name"
    values = ["debian-11-amd64-*"]
  }
}

resource "aws_launch_template" "nodes" {
  name = "${var.cluster_name}_nodes_lt"

  block_device_mappings {
    device_name = "/dev/xvda"

    ebs {
      volume_size = 100
    }
  }

  network_interfaces {
    security_groups = [aws_security_group.nodes.id]
    associate_public_ip_address = true
    subnet_id = aws_subnet.subnet.id
  }

  instance_type = var.node_instance_type
  image_id = data.aws_ami.ec2_instance.id

  key_name = var.node_ssh_key_name
}

resource "aws_autoscaling_group" "nodes" {
  name = "${var.cluster_name}_nodes"

  desired_capacity   = var.cluster_node_count
  max_size           = var.cluster_node_count
  min_size           = var.cluster_node_count

  availability_zones = var.node_availability_zone

  launch_template {
    id      = aws_launch_template.nodes.id
    version = "$Latest"
  }

  tag {
    key = "Application"
    value = "madara"
    propagate_at_launch = true
  }
}

data "aws_instances" "instances" {
  depends_on = [aws_autoscaling_group.nodes]
  instance_tags = {
    Application = "madara"
  }

  instance_state_names = ["running"]
}

resource "local_file" "instances-ips" { 
  filename = "${path.module}/../../ansible/nodes.ips"
  content = jsonencode(data.aws_instances.instances.public_ips)
}
