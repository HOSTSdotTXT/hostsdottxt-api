variable "PKGVER" {
    default = "0.1.0"
}

group "default" {
    targets = ["deb"]
}

target "bin" {
    dockerfile = "Dockerfile"
    target = "bin"
    output = ["artifacts"]
}

target "deb" {
    dockerfile = "Dockerfile"
    target = "deb"
    output = ["artifacts"]
    args = {
        VERSION="${PKGVER}"
    }
}
