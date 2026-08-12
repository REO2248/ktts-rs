
BOOLT IsKVoiceJaribCheon(BYTE bPumsa)

{
  if ((((3 < (byte)(bPumsa - 0x30)) && (bPumsa != '9')) && (bPumsa != '<')) &&
     ((bPumsa != 'F' && (bPumsa != ':')))) {
    return (uint)(bPumsa == 'H');
  }
  return 1;
}

