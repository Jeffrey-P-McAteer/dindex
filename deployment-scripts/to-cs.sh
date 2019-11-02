#!/bin/sh

cd "$(dirname "$(realpath "$0")")";
cd ..

cargo build --release || exit 1

ssh cs 'mkdir -p secure_html/cgi-bin/ ; rm secure_html/cgi-bin/dindex.cgi ; chmod a+rx secure_html/cgi-bin/'
scp 'target/release/dindex' cs:'secure_html/cgi-bin/dindex.cgi'
ssh cs 'chmod a+rx secure_html/cgi-bin/dindex.cgi ; chmod a+r secure_html/.htaccess'
ssh cs 'cat > secure_html/.htaccess' <<EOF
Options +ExecCGI
AddHandler cgi-script .cgi .pl
EOF

curl -v 'http://cs.odu.edu/~jmcateer/cgi-bin/dindex.cgi'

# Fails b/c the CS server uses a 10-year-old copy of libc

