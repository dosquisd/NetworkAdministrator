use network_administrator::ads::response::remove_ad_scripts;

fn main() {
    let html = r#"
    <!DOCTYPE html>
    <html lang="es">
    <head>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Canal 24 En Vivo | Tarjeta Roja</title>
    <meta name="description" content="Ver Canal 24 en vivo y gratis por internet | Tarjeta Roja"/>
    <meta name="distribution" content="global"/>
    <meta content="TARJETA ROJA" name="author"/>
    <meta name="robots" content="all"/>
    <meta http-equiv="Content-Language" content="es"/>
    <link href="/layout2.css" rel="stylesheet" type="text/css" />
    <link rel="shortcut icon" href='/favicon.ico'> 
    <script src="https://ajax.googleapis.com/ajax/libs/jquery/1.7.1/jquery.min.js"></script>
    <!-- Google tag (gtag.js) -->
    <script async src="https://www.googletagmanager.com/gtag/js?id=G-RF152VT1J6"></script>
    <script>
      window.dataLayer = window.dataLayer || [];
      function gtag(){dataLayer.push(arguments);}
      gtag('js', new Date());
      gtag('config', 'G-RF152VT1J6');
    </script>
    <script id="aclib" type="text/javascript" src="//acscdn.com/script/aclib.js"></script>
    <script type="text/javascript"> aclib.runPop({ zoneId: '10289202', }); </script>
    </head>

    <body>
    <header><a href="/"><img src="/logo.png" height="35" width="350"/></a><a href="/programacion.php"><span class="title2">PROGRAMACION</span></a><a href="/legal.php"><span class="title3">AVISO LEGAL</span></a></header>
    <div class="container">
    <div class="main">
    <div class="title">Canal 24 En Vivo</div>
    <div class="embed-responsive"><iframe src="https://www.capoplay.net/canal24.php" allow="encrypted-media" width="100%" height="100%" frameborder="0" scrolling="no" allowfullscreen="true"></iframe></div>
    </div>
    <footer><a href="/">TARJETA ROJA | Rojadirecta TV | Deportes En Vivo<span class="icon-dot"></span></a> | © 2025 Diseño por tarjetarojaenvivo.club</footer>
    </div>

    <!-- Case 1 Google: Ad code for non-personalized ads -->
    <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pubxxx" crossorigin="anonymous"></script>
    <script>(adsbygoogle=window.adsbygoogle||[]).pauseAdRequests=1;</script>

    <!-- Case 2 Google: Ad code for personalized ads -->
    <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pubxxx" crossorigin="anonymous"></script>
    <script>(adsbygoogle=window.adsbygoogle||[]).requestNonPersonalizedAds=1;</script>

    <!-- Case 3 Google: Auto ads: ad code for non-personalized ads -->
    <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pubxxx" crossorigin="anonymous"></script>
    <script>
    (adsbygoogle=window.adsbygoogle||[]).requestNonPersonalizedAds=1;
    (adsbygoogle=window.adsbygoogle||[]).push({google_ad_client: "ca-pubxxx", enable_page_level_ads: true});
    </script>

    <!-- Case 1: Simple -->
    <script>
      (adsbygoogle = window.adsbygoogle || []).push({});
    </script>

    <!-- Case 2: With config -->
    <script>
      (window.adsbygoogle = window.adsbygoogle || []).push({
        google_ad_client: "ca-pub-123456",
        enable_page_level_ads: true
      });
    </script>

    <!-- Case 3: Complex multiline -->
    <script type="text/javascript">
      var googletag = googletag || {};
      (window.adsbygoogle = window.adsbygoogle || []).push({});
    </script>

    <script>console.log('Injected script by Network Administrator');</script>

    </body>
    </html>
    "#;

    println!("{}", remove_ad_scripts(html));
}

/*
<!DOCTYPE html>
<html lang="es">
<head>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Canal 24 En Vivo | Tarjeta Roja</title>
<meta name="description" content="Ver Canal 24 en vivo y gratis por internet | Tarjeta Roja"/>
<meta name="distribution" content="global"/>
<meta content="TARJETA ROJA" name="author"/>
<meta name="robots" content="all"/>
<meta http-equiv="Content-Language" content="es"/>
<link href="/layout2.css" rel="stylesheet" type="text/css" />
<link rel="shortcut icon" href='/favicon.ico'>
<script src="https://ajax.googleapis.com/ajax/libs/jquery/1.7.1/jquery.min.js"></script>
<!-- Google tag (gtag.js) -->

<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'G-RF152VT1J6');
</script>

<script type="text/javascript"> aclib.runPop({ zoneId: '10289202', }); </script>
</head>

<body>
<header><a href="/"><img src="/logo.png" height="35" width="350"/></a><a href="/programacion.php"><span class="title2">PROGRAMACION</span></a><a href="/legal.php"><span class="title3">AVISO LEGAL</span></a></header>
<div class="container">
<div class="main">
<div class="title">Canal 24 En Vivo</div>
<div class="embed-responsive"><iframe src="https://www.capoplay.net/canal24.php" allow="encrypted-media" width="100%" height="100%" frameborder="0" scrolling="no" allowfullscreen="true"></iframe></div>
</div>
<footer><a href="/">TARJETA ROJA | Rojadirecta TV | Deportes En Vivo<span class="icon-dot"></span></a> | © 2025 Diseño por tarjetarojaenvivo.club</footer>
</div>

<!-- Case 1 Google: Ad code for non-personalized ads -->



<!-- Case 2 Google: Ad code for personalized ads -->



<!-- Case 3 Google: Auto ads: ad code for non-personalized ads -->



<!-- Case 1: Simple -->


<!-- Case 2: With config -->


<!-- Case 3: Complex multiline -->


<script>console.log('Injected script by Network Administrator');</script>

</body>
</html>
*/
