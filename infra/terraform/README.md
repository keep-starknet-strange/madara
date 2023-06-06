# Terraform

## Description

This folder contains Terraform scripts that are organized into modules for
managing infrastructure on a particular cloud provider (AWS, Google Cloud,
Azure). Terraform allows you to define your infrastructure as code, which can
then be shared and re-used.

## Usage

1. Create a service account user in your cloud provider with sufficient
   permissions.
2. Add the credentials to the `creds.tfvars.example` file and rename it to
   `creds.tfvars`
3. Use the `run.sh` script to deploy the infrastructure
