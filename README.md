抽象構文木を再帰的に辿るだけのシンプルな実装のスクリプト言語です。
GCがない代わりに参照が存在しません。

exp.pestに構文が書いてあります。
生成される構文木のデータ構造についてはast.rsを見てね！


再帰呼び出しを使った階乗を求めるサンプルコード:
```
f main(): factrial(10);
f factrial(n): if n = 0 -> 1 else -> n * factrial(n - 1);

```

イテレータを使った階乗を求めるサンプルコード:
```
f main(): {
    let fact: 1;
    for i in [1, 2, 3, 4]: {
        fact <<- fact * i;
    };
} => fact;

```

はろーわーるど！:
```
f main(): "Hello World!";
```
