#!/bin/sh

if [ -z "$1" ]; then
    echo "Usage: $0 <aws>"
    exit 1
fi

cd $(dirname $0)

terraform -chdir=$1 init
terraform -chdir=$1 apply --var-file=$1.tfvars --var-file=../creds.tfvars
