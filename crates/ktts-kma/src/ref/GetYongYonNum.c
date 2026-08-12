

BOOLT GetYongYonNum(WCHART *pwVstr,BYTE *pbArray,int *pnSize)

{
  size_t sVar1;
  BOOLT BVar2;
  BOOLT BVar3;
  uint uVar4;
  BOOLT fVerb;
  
  sVar1 = Wcslen(pwVstr);
  uVar4 = 0;
  if (1 < sVar1) {
    BVar2 = ExceptYongYon(pwVstr,pbArray,pnSize);
    uVar4 = 1;
    if (BVar2 == 0) {
      BVar2 = GetVerbNum(pwVstr,pbArray,pnSize);
      BVar3 = GetHongYongSaNum(pwVstr,pbArray,pnSize);
      uVar4 = (uint)(BVar3 != 0 || BVar2 != 0);
    }
  }
  return uVar4;
}

