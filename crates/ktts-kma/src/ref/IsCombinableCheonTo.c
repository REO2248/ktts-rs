

BOOLT IsCombinableCheonTo(char *pchWord,BOOLT bNew,LPSMORPHCANDINFO psCand)

{
  uint uVar1;
  BOOLT BVar2;
  BOOLT BVar3;
  char *pcVar4;
  
  if (bNew == 0) {
    BVar2 = IsClosedSyllableKRoot(pchWord);
    BVar3 = IsKCheonTo(psCand->chPumsa);
    if (BVar3 == 0) {
      BVar3 = IsKYongonTo(psCand->chPumsa);
      if (BVar3 != 0) {
        if ((psCand->unToInfo & 0x20) == 0) {
          return 0;
        }
        if (BVar2 != 0) {
          pcVar4 = strchr("bcdfghjklmnpqrstvxz",(int)psCand->schMorphStr[0]);
          return (uint)(pcVar4 == (char *)0x0);
        }
      }
    }
    else {
      uVar1 = psCand->unToInfo;
      if (uVar1 != 0) {
        if ((BVar2 != 0) && ((uVar1 & 0x400) != 0)) {
          if (psCand->chPumsa != 'Y') {
            return 0;
          }
          return (uint)(PTR_gschMorphCandStr_00095164[*(int *)PTR_gnMorphCandStr_000952f8] == 'L');
        }
        return (uint)(BVar2 == 0) & uVar1 >> 0xb ^ 1;
      }
    }
  }
  return 1;
}

