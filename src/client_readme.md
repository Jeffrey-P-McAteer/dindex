Usage: dindex-client command [command options]

While raw objects are fine, there are common schemas (a webpage, someone's phone number, etc.)
which will be generated by using flags (ex see :webpage below).

  dindex-client publish {{raw object}}
  dindex-client publish :webpage url 'title' 'description'

  dindex-client query {{query object}}
  dindex-client query :webpage 'url-regex' 'title-regex' 'description-regex'
  dindex-client query --oldsearch keyword1 keyword2 keyword3

Less-common comands:

  dindex-client --publish-site-pages http://example.com [--depth N]
  
    This downloads the given webpage and creates `webpage` records
    for the page and up to N pages deep it links to (defaulting to 0).
    These records are then published to all upstream_resolvers from your config file.

  dindex-client publish [--signed]
    
    the `--signed` flag is available for all publish commands and wraps given objects
    like so:
    
      {{
        "title": "My new blog post!",
        "url":"http://example.org/post.html"
      }}
    becomes
      {{
        "title": "My new blog post!",
        "url":"http://example.org/post.html",
        "title-sig": "meh-bytes-go-here",
        "url-sig": "meh-bytes-go-here",
        "signature-type":"RSASSA-PSS",
        "signature-public-key":"MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCZsiTSNpZ4JHAJm6iTJnrU0j8QkvX8c8/x9aY5mpo9nYm+0IsXG0L9M+OYnzeN9nuiph6zAaG08nlJ7iCHUyGT5lebxuRFa6RiC6hpd5Q/9REi7xQbRBhlolI+Bw0itKkL43KHdtYff5164/rROSiYHnAy8W01b70wNy9G3uqOGQIDAQAB"
      }}
    
    Requires `identity_private_key` and `identity_public_key` set in your config file.
  
  dindex-client query [--encrypted]
    
    TODO signed queries sound dumb, what is useful is _encrypted_ queries to protect the neighborhood perverts.

Raw Objects

  Raw objects are key/value records which comprise the smallest piece of information dindex manipulates.
  Keys _should_ be all lowercase and < 100 characters.
  Values _should_ be UTF-8 and < 6000 characters.
  Each object _should_ have 10 or fewer key/value pairs.
  
  Servers make no promise to store given objects and may reject them for any reason.
  Servers _should_ reply to rejected objects with a message containing "type": "ephemeral" along with a message explaining why the record was rejected.
  
  Raw object do not support any kind of direct encryption or authentication.
  If this is desired simply use a key to provide the capability; ex:
  
    {{
      "title":"Secret url",
      "encrypted-url":"Bge9a1cdzhnH9xG3nsBcmbJt7JYCeNkm+YJWKBu5d3mwGlTKldIdcWO4FasxmkHU",
      "encryption-type":"AES-ECB-128"
    }}
  
    {{
      "title":"Authentic url signed by Oracle",
      "url": "http://example.org/java-official-download.html",
      "signed-url":"ddtC7l+iSEhglhXB1R908yjgzdJ5zNXb4g8SJtLsftjbkCqcn/9GEL5uCnLSy/DiB+TqjNWDNsDOUZBgg5Nj1wid4hPoKwGaEJjnghuStwUtt4G9UFDjB624CSBCulwj9jOsjARwWLeprkrWMrI+t5XD56ywc6ush4KN1V5PTzQ=",
      "signature-type":"RSASSA-PSS",
      "signature-public-key":"MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCZsiTSNpZ4JHAJm6iTJnrU0j8QkvX8c8/x9aY5mpo9nYm+0IsXG0L9M+OYnzeN9nuiph6zAaG08nlJ7iCHUyGT5lebxuRFa6RiC6hpd5Q/9REi7xQbRBhlolI+Bw0itKkL43KHdtYff5164/rROSiYHnAy8W01b70wNy9G3uqOGQIDAQAB"
    }}
  
  Notice how using the "url" parameter has an added bonus: signed records will appear in
  queries which do not care about how the data is signed.
  
  Additionaly, the "signed-url" may be used in queries to only show results which are signed.
  For example the following query object will match the above object:
  
    {{
      "title": ".*Oracle.*",
      "signed-url": ".*"
    }}
  
  
Query Objects

  Query objects are just like raw objects, but the given value for each key is used as a search regex.
  
  example: for a server which knows about the following records:
     {{"title": "US lands on moon", "url": "http://example.org/article01.html"}}
     {{"title": "Farmers use land for memorial", "url": "http://example.com/article02.html"}}
     {{"name": "Jeffrey McAteer", "phone": "555-444-2222"}}
  a client giving the following query object:
     {{"title": ".*land.*"}}
  would receive:
     {{"title": "US lands on moon", "url": "http://example.org/article01.html"}}
     {{"title": "Farmers use land for memorial", "url": "http://example.com/article02.html"}}
  
  a client giving the following query object:
     {{"url": ".*example.org.*"}}
  would receive:
     {{"title": "US lands on moon", "url": "http://example.org/article01.html"}}
    
  a client giving the following query object:
     {{"name": "Jeffrey McAteer"}}
  would receive:
     {{"name": "Jeffrey McAteer", "phone": "555-444-2222"}}