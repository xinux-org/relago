# Relago

## Hamidulloh

#### Journal Fetcher
- Journaldan kerakli data larni bizga kerakli structda olib kelish
- Journal handlerga barcha ma'lumotlar yetkaziladi qayta ishlangan holatda

#### Journal Handler (Relago)
- Journal Fetcherni ishga tushiradi.
- View/modalga bo'lib o'tgan errorlarni process qilib, ular haqida xabar beriladi.
- So'ng qabul qilib olingan ma'lumotlar db_file - <custom db type || sqlite> tipli faylga yozib olinadi. (Ehtimoliy binary file)
- Errorlar haqida GNOME notificationsga xabar beriladi, xuddi "oops!" kabi

#### Report Lib (ehtimoliy)
- View/modaldan olingan ma'lumotlarni report uchun struct ni to'g'irlab, kerakli field larni qo'shib serverga report qiladi.
- Bundan tashqari, db_file'ni ham serverga report qiladi.
(Server qani deb hayron bo'lmaymiz, Ahmad aka hal qilarkan serverni)

## Kamron

#### View/modal
- View/modal hamma errorlarni yoziladigan GTK app orqali ko'rsatiladi.
- Buning ishga tushishi avtomatik amalga oshiriladi.
- GTK app'da ba'zi optional checkboxes'dagi configlarni o'zining config.toml fayliga yozib qo'yadi
- Details button bosilganda current errors plain text holatda ko'rsatiladi.

#### On-startup Check
- Relago start bo'layotgan davrda journal'dan kernelga aloqador hamma loglar fetch qilib olinadi. (Voobshe oson ekan, tayyor ekan, chopish kerak ekan.)
