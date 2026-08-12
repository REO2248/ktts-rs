

BOOLT IsBadTimTo(BYTE bTag,char *pchStr)

{
  BOOLT BVar1;
  uint uVar2;
  
  BVar1 = IsTo(bTag);
  uVar2 = 0;
  if (BVar1 != 0) {
    uVar2 = (uint)((byte)(*pchStr + 0xbfU) < 0x1a);
  }
  return uVar2;
}

