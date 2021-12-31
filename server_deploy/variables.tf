variable "app_name" {
    type = string
    description = "Name of deployed application."
    default = "nat-punch-server"
}

variable "git_credentials" {
    type = string
    description = "Git credential string to allow for cloning repository."
}

variable "static_ip" {
    type = string
    description = "Static IP for instance"
    default = "34.67.17.122"
}