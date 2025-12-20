# Scripts

## Pi-hole

[Pi-hole](https://pi-hole.net/) fue una de las fuentes que tome como inspiración para motivarme a desarrollarme mi propio AdBlocker, en realidad funcionamos de manera muy diferente, Pi-hole funciona principalmente como un servidor DNS, lo cual tiene sus ventajas de ser más ligero y evitar varios conflictos innecesarios respecto a HTTP/HTTPS o los certificados TLS (`certificate pinning` entre tantos problemas). Sin embargo, yo quería hacer algo más, quería poder modificar el contenido de la respuesta del servidor (el HTML) antes de que le llegue al cliente (mi computador por ejemplo), y fue que decidí hacer este proxy.

Aun así, aunque se puedan ver como "competencia" (no me interesa eso realmente), se pueden complementar uno al otro. El proxy puede quedar como servicios usado en el navegador principalmente, y Pi-hole como servicio usado para ser utilizado en toda la red, dispositivos y aplicaciones moviles por ejemplo. Esa es la gran ventaja que tiene el servidor DNS.

En [run-pi-hole.sh](./run-pi-hole.sh) está un script de Bash para ejecutar Pi-hole con Docker en la Raspberry PI.
