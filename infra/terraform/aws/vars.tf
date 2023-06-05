variable "aws_access_key" {
  type        = string
  description = "AWS access key"
}

variable "aws_region" {
  type        = string
  description = "AWS region"
}

variable "aws_secret_key" {
  type        = string
  description = "AWS secret key"
}

variable "cluster_name" {
  type = string
  description = "Name for the cluster for AWS resources to have prefixed"
}

variable "cluster_node_count" {
  type = string
  description = "Number of nodes to launch"
}

variable "net_availability_zone" {
  type = string
  description = "Availability zone for subnet"
}

variable "net_subnet_cidr" {
  type = string
  description = "Subnet CIDR"
}

variable "net_vpc_cidr" {
  type = string
  description = "VPC CIDR"
}

variable "node_availability_zone" {
  type = list(string)
  description = "Availability zone for nodes"
}

variable "node_instance_type" {
  type = string
  description = "AWS EC2 instance type for the nodes"
}

variable "node_ssh_key_name" {
  type = string
  description = "Existing AWS SSH key to add to the launched EC2 instances"
}
