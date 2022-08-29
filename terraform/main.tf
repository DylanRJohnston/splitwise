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

module "s3_bucket" {
  source = "terraform-aws-modules/s3-bucket/aws"

  bucket_prefix = "terraform-state"
  acl           = "private"

  versioning = {
    enabled = true
  }

}

module "dynamodb_table_integration_test" {
  source = "terraform-aws-modules/dynamodb-table/aws"

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
  source = "terraform-aws-modules/dynamodb-table/aws"

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

resource "aws_secretsmanager_secret" "splitwise" {
  name_prefix = "splitwise-"
}

module "lambda_function" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "splitwise"
  handler       = "bootstrap"
  description   = "Reconciles Splitwise and YNAB"
  runtime       = "provided.al2"
  architectures = ["x86_64"]
  package_type  = "Zip"
  timeout       = 20
  publish       = true

  cloudwatch_logs_retention_in_days = 7

  environment_variables = {
    RUST_BACKTRACE          = 1
    AWS_SECRETS_MANAGER_ARN = aws_secretsmanager_secret.splitwise.arn
  }

  create_package         = false
  local_existing_package = "../result"

  attach_policy_statements = true
  policy_statements = {
    dynamodb = {
      effect = "Allow"
      actions = [
        "dynamodb:BatchWriteItem",
        "dynamodb:BatchGetItem",
        "dynamodb:PutItem",
        "dynamodb:GetItem",
        "dynamodb:Query",
        "dynamodb:Scan",
      ]
      resources = [module.splitwise_dynamodb_table.dynamodb_table_arn]
    }
    secrets_manager = {
      effect    = "Allow"
      actions   = ["secretsmanager:GetSecretValue"]
      resources = [aws_secretsmanager_secret.splitwise.arn]
    }
  }
  allowed_triggers = {
    eventbridge = {
      principal  = "events.amazonaws.com"
      source_arn = module.eventbridge.eventbridge_rule_arns.splitwise
    }
  }
}

module "eventbridge" {
  source = "terraform-aws-modules/eventbridge/aws"

  create_bus = false

  rules = {
    splitwise = {
      description         = "Trigger for a Lambda"
      schedule_expression = "rate(1 hour)"
    }
  }

  targets = {
    splitwise = [
      {
        name  = "lambda-splitwise-cron"
        arn   = module.lambda_function.lambda_function_arn
        input = jsonencode({})
      }
    ]
  }
}

resource "aws_iam_openid_connect_provider" "github" {
  url = "https://token.actions.githubusercontent.com"

  client_id_list = ["sts.amazonaws.com"]

  thumbprint_list = ["15e29108718111e59b3dad31954647e3c344a231"]
}

resource "aws_iam_role" "github" {
  name_prefix = "github"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRoleWithWebIdentity"
        Effect = "Allow"
        Sid    = ""
        Principal = {
          Federated = aws_iam_openid_connect_provider.github.arn
        }
        Condition = {
          StringEquals = {
            "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com",
            "token.actions.githubusercontent.com:sub" = "repo:DylanRJohnston/splitwise:environment:production"
          }
        }
      }
    ]
  })

  managed_policy_arns = ["arn:aws:iam::aws:policy/AdministratorAccess"]
}
