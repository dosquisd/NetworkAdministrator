# Scripts

## Pi-hole

[Pi-hole](https://pi-hole.net/) was one of the sources that inspired me to develop my own AdBlocker. Actually, we work very differently. Pi-hole primarily functions as a DNS server, which has advantages of being lighter and avoiding several unnecessary conflicts regarding HTTP/HTTPS or TLS certificates, e.g., `certificate pinning`. However, I wanted to do something more. I wanted to be able to modify the server response content (the HTML) before it reaches the client (e.g., my computer), and that's when I decided to make this proxy.

Even so, although they can be seen as "competitors" (I don't really care about that to be honest), they can complement each other. The proxy can serve as a service used mainly in the browser, and Pi-hole as a service used for the entire network, mobile devices and applications for example. That is the great advantage that the DNS server has.

In [run-pi-hole.sh](./run-pi-hole.sh) there is a Bash script to run Pi-hole with Docker on the Raspberry PI.
