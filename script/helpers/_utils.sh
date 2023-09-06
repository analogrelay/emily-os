error() {
    echo "$@" >&2
}

fatal() {
    error "$@"
    exit 1
}

properly_configured() {
    if [ ! -d ".config" ] || [ ! -f ".config/arch" ] || [ ! -f ".config/board" ]; then
        return 1
    fi
}