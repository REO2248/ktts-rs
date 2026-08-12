
BOOLT YongYonAdd(BYTE *pbArray,int *pnSize,BYTE bByte)

{
  if (*pnSize < 2) {
    pbArray[*pnSize] = bByte;
    *pnSize = *pnSize + 1;
  }
  return 1;
}

