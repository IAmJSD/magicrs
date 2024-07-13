// Defines a regex for RFC 1123 compliant domain names or IP addresses.
pub const DOMAIN_OR_IP_REGEX: &str = "^(([a-zA-Z]|[a-zA-Z][a-zA-Z0-9\\-]*[a-zA-Z0-9])\\.)*([A-Za-z]|[A-Za-z][A-Za-z0-9\\-]*[A-Za-z0-9])$";

// Defines the FTP description for URL rewrites since it is fairly long.
pub const URL_FTP_REWRITE_DESCRIPTION: &str = concat!(
    "The string to rewrite the URL to. In this URL, you can use `$hostname` to represent the hostname, ",
    "`$folder_path` to represent the folder path, and `$filename` to represent the filename. The default ",
    "is `https://$hostname$folder_path/$filename`.",
);
