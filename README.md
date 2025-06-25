# Redact Personal Identifiable information as a service

This is a simple microservice that will accept a JSON string as input to the `/redact` endpoint and return a JSON string with the PII redacted based on regular expressions stored as JSON.

Removed the psutil functionality to do health checks due to conflict with other dependencies as psutil uses a dependency which is not maintained anymore, causing the conflict.

Will add the health check back when a suitable replacement of psutil is found 😊.
