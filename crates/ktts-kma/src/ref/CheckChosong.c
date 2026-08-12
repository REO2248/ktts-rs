
int CheckChosong(char unMoum)

{
  int iVar1;
  
  iVar1 = 0;
  if ((unMoum != '\0') &&
     ((((unMoum == '\x03' || (unMoum == '\x01')) || (unMoum == '\r')) || (iVar1 = 2, unMoum == '\t')
      ))) {
    return 1;
  }
  return iVar1;
}

