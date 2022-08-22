terraform {
  backend "s3" {
    bucket = "terraform-state20220821082045303200000001"
    key    = "splitwise"
    region = "ap-southeast-2"
  }
}

provider "aws" {
  region = "ap-southeast-2"
}

# module "s3_bucket" {
#   source = "terraform-aws-modules/s3-bucket/aws"

#   bucket = "terraform-state"
#   acl    = "private"

#   versioning = {
#     enabled = true
#   }

# }


module "dynamodb_table_integration_test" {
  source   = "terraform-aws-modules/dynamodb-table/aws"

  name     = "splitwise_integration_test"
  hash_key = "id"

  attributes = [
    {
      name = "id"
      type = "S"
    }
  ]

  tags = {
    Terraform   = "true"
    Environment = "test"
  }
}

module "splitwise_dynamodb_table" {
  source   = "terraform-aws-modules/dynamodb-table/aws"

  name     = "splitwise"
  hash_key = "id"

  attributes = [
    {
      name = "id"
      type = "S"
    }
  ]

  tags = {
    Terraform   = "true"
    Environment = "prod"
  }
}