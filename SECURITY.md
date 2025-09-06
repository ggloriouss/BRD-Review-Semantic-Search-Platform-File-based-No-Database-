# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x     | :white_check_mark: |
| 0.x     | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it by opening an issue or contacting the maintainers directly.

- Issues: [GitHub Issues](https://github.com/ggloriouss/BRD-Review-Semantic-Search-Platform-File-based-No-Database-/issues)
- Email: nipapornphomsak@gmail.com

We aim to respond to security reports within 72 hours.  
Once a vulnerability is confirmed, we will provide updates on mitigation and patch timelines.

## Security Practices

- All data is stored in append-only files; no direct deletion or modification is supported.
- No external database is used.
- Review input is validated for required fields and rating range.
- Docker containers run with least privilege.
- Dependencies are regularly updated.

Please see the LICENSE file for legal