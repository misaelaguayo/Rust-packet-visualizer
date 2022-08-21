use fraction;

fn main(){
    type F = fraction::Fraction;
    let result = F::from(1) / F::from(3);
    let new = fraction::Fraction::new(1u8, 2u8);
    println!("{} {}", new.num, new.den);
}
