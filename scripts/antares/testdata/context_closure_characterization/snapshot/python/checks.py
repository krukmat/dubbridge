def validate_scope(path: str) -> bool:
    return path.endswith(".py") or path.endswith(".rs")
