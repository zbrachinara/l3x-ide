//TODO: replace with maybe_uninit


struct ConstVec<T, const L: usize> {
    data: [Option<T>; L]
}
impl<T, const L: usize> ConstVec<T, {L}> {
    const n: Option<T>=None;
    const fn push(self, value: T) -> ConstVec<T, {L+1}> {
        self.insert(value, L)
    }
    const fn pop(self) -> (ConstVec<T,{L-1}>,T) {
        self.take(L-1)
    }
    const fn take(mut self, index: usize) -> (ConstVec<T, {L-1}>,T){
        assert!(index<L);
        let mut res = [Self::n; L-1];
        let mut i = 0;
        while i<index {
            std::mem::forget(std::mem::replace(&mut res[i],self.data[i].take()));
            i+=1;
        }
        let resT = self.data[index].take().unwrap();
        i+=1;
        while i<L {
            std::mem::forget(std::mem::replace(&mut res[i],self.data[i].take()));
            i+=1;
        }
        std::mem::forget(self);
        (ConstVec{data: res},resT)
    }
    const fn insert(mut self, value: T, index: usize) -> ConstVec<T, {L+1}>{
        assert!(index<=L);
        let mut res = [Self::n; L+1];
        let mut i = 0;
        while i<index {
            std::mem::forget(std::mem::replace(&mut res[i],self.data[i].take()));
            i+=1;
        }
        std::mem::forget(std::mem::replace(&mut res[index],Some(value)));
        i+=1;
        while i<=L {
            std::mem::forget(std::mem::replace(&mut res[i],self.data[i-1].take()));
            i+=1;
        }
        std::mem::forget(self);
        ConstVec{data: res}
    }
    const fn get(&self, index:usize) -> &T {
        self.data[index].as_ref().unwrap()
    }
}
impl<T> ConstVec<T,0> {
    const fn new() -> ConstVec<T,0>{
        ConstVec{data: [None;0]}
    }
}
/*impl<const L: usize> ConstVec<u64, L>{
    const fn drainToSum(self) -> u64{
        if L==0 {
            return 0
        }
        let (ns,val)=self.pop();
        val+ns.drainToSum()
    }
}*/
impl ConstVec<u64, 0> {
    const fn drainToSumZero(self) -> u64 {
        0
    }
}
/*fn constVecToStr<T, const L: usize>(a: ConstVec<T,L>) -> String {
    let mut s=String::from("]");
    if L>0 {
        s+=&a[0].to_string()
    }
    let mut i = 1;
    while i<N {
        s+=",";
        s+=&a[i].to_string();
        i+=1;
    }
    s+"]"
}*/

#[test]
fn test() {
    const c: ConstVec<u64,4> = const {let a = ConstVec::<u64,0>::new();
    let b=a.push(0).push(1).push(2).push(3);b};

}