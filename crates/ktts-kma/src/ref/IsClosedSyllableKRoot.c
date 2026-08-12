

BOOLT IsClosedSyllableKRoot(char *pchStr)

{
  size_t sVar1;
  
  sVar1 = strlen(pchStr);
  return (uint)((byte)(pchStr[sVar1 - 1] + 0xbfU) < 0x1a || pchStr[sVar1 - 1] == '*');
}

