

BOOLT ExceptYongYon(WCHART *pwVstr,BYTE *pbArray,int *pnSize)

{
  WCHART WVar1;
  size_t sVar2;
  BOOLT BVar3;
  WCHART WVar4;
  
  sVar2 = Wcslen(pwVstr);
  if (1 < (int)sVar2) {
    WVar4 = 0;
    WVar1 = pwVstr[sVar2 - 2];
    if (sVar2 != 2) {
      WVar4 = pwVstr[sVar2 - 3];
    }
    if (WVar1 == 0xc54a) {
      YongYonAdd(pbArray,pnSize,'\r');
      BVar3 = YongYonAdd(pbArray,pnSize,',');
      return BVar3;
    }
    if (WVar1 == 0xc788) {
      BVar3 = YongYonAdd(pbArray,pnSize,'\x1f');
      return BVar3;
    }
    if (WVar1 == 0xc5c6) {
      BVar3 = YongYonAdd(pbArray,pnSize,' ');
      return BVar3;
    }
    if ((WVar4 == 0xacc4) && (WVar1 == 0xc2dc)) {
      BVar3 = YongYonAdd(pbArray,pnSize,'!');
      return BVar3;
    }
    if (WVar1 == 0xc8b8) {
      BVar3 = YongYonAdd(pbArray,pnSize,'*');
      return BVar3;
    }
    if (WVar1 == 0xd1b1) {
      BVar3 = YongYonAdd(pbArray,pnSize,'\x17');
      return BVar3;
    }
  }
  return 0;
}

