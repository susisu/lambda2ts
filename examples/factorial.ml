let Id x = x;
let Const x y = x;
let Fix f = (fun x -> f (fun y -> x x y)) (fun x -> f (fun y -> x x y));

let One f x = f x;
let Mul a b = fun f x -> a (b f) x;
let Pred n = fun f x -> n (fun g h -> h (g f)) (Const x) Id;

let True = Const;
let False x y = y;
let IsZero n = n (Const False) True;

let Factorial n =
  let f = Fix (fun f n r ->
    (IsZero n) r (f (Pred n) (Mul n r))
  ) in f n One;
