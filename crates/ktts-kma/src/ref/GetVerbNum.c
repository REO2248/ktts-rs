

BOOLT GetVerbNum(WCHART *pwVstr,BYTE *pbArray,int *pnSize)

{
  int iVar1;
  size_t sVar2;
  BOOLT BVar3;
  int iVar4;
  WCHART wWan;
  int in_GS_OFFSET;
  BYTE bByte;
  WCHART wCX1;
  char schKX1 [4];
  char schKX0 [4];
  
  iVar1 = *(int *)(in_GS_OFFSET + 0x14);
  sVar2 = Wcslen(pwVstr);
  if ((int)sVar2 < 2) {
    wWan = 0;
    wCX1 = 0;
  }
  else {
    wWan = pwVstr[sVar2 - 2];
    if (sVar2 == 2) {
      wCX1 = 0;
    }
    else {
      wCX1 = pwVstr[sVar2 - 3];
    }
  }
  ConvUniCodeToCVC(wWan,schKX0);
  ConvUniCodeToCVC(wCX1,schKX1);
  if (schKX0[2] == '\0') {
    if (schKX0[1] == '\x01') {
      if (wWan == 0xac00) {
        bByte = '\t';
      }
      else if (wWan == 0xd558) {
        bByte = '\x14';
      }
      else {
        bByte = '\x01';
      }
    }
    else if (schKX0[1] == '\x05') {
      if (wCX1 == 0xadf8) {
        if (wWan == 0xb7ec) goto LAB_00075906;
      }
      else if ((wCX1 == 0xc5b4) && (wWan == 0xca4c)) {
LAB_00075906:
        bByte = '\x15';
        goto LAB_00075414;
      }
      bByte = '\x02';
    }
    else if ((((schKX0[1] == '\a') || (schKX0[1] == '\x02')) || (schKX0[1] == '\x06')) ||
            (schKX0[1] == '\f')) {
      bByte = '\x03';
    }
    else if (((byte)(schKX0[1] - 0x10U) < 2) || (schKX0[1] == '\x14')) {
      bByte = '\x04';
    }
    else if (schKX0[1] == '\t') {
      if (wWan == 0xc624) {
        bByte = '\n';
      }
      else {
        bByte = '\x05';
      }
    }
    else if (schKX0[1] == '\x12') {
      bByte = '\x06';
    }
    else if (schKX0[1] == '\x0e') {
      if (wWan == 0xd478) {
        bByte = '\x13';
      }
      else {
LAB_0007584a:
        bByte = '\a';
      }
    }
    else if (schKX0[1] == '\x15') {
      bByte = '\b';
    }
    else {
      if (schKX0[1] != '\x13') goto LAB_00075530;
      if (wWan == 0xb974) {
        if (wCX1 == 0xc774) {
          bByte = '\x12';
        }
        else if (wCX1 == 0xb530) {
LAB_00075a41:
          bByte = '\x0e';
        }
        else {
          if (wCX1 == 0xb7ec) goto LAB_00075833;
          if (wCX1 == 0xce58) goto LAB_0007584a;
          iVar4 = CheckChosong(schKX1[1]);
          if (iVar4 == 1) {
            bByte = '\x10';
          }
          else {
            bByte = '\x11';
          }
        }
      }
      else {
        if (wWan == 0xadf8) goto LAB_0007584a;
        if ((wWan != 0xc4f0) && (sVar2 != 2)) {
          iVar4 = CheckChosong(schKX1[1]);
          if (iVar4 == 1) goto LAB_00075a41;
          iVar4 = CheckChosong(schKX1[1]);
          BVar3 = 1;
          if (iVar4 != 2) goto LAB_00075426;
        }
LAB_00075833:
        bByte = '\x0f';
      }
    }
  }
  else if (schKX0[2] == '\x11') {
    if (wWan == 0xb3d5) {
      bByte = '\x16';
    }
    else if (((wWan == 0xcb59) || (wWan == 0xbd59)) || (wWan == 0xc635)) {
      bByte = '\x18';
    }
    else {
      if (wWan != 0xc90d) {
        if (wWan == 0xaf3d) {
          if (wCX1 == 0xb2c8) goto LAB_000757a0;
        }
        else {
          if (((schKX0[0] == '\t') || (schKX0[0] == '\f')) ||
             ((schKX0[0] == '\v' || (schKX0[0] == '\r')))) goto LAB_000753f7;
          if ((((schKX0[0] == '\x03') || (schKX0[0] == '\x01')) ||
              ((schKX0[0] == '\x04' ||
               (((((schKX0[0] == '\x06' || (schKX0[0] == '\a')) || (schKX0[0] == '\b')) ||
                 ((schKX0[0] == '\n' || (schKX0[0] == '\x0f')))) || (schKX0[0] == '\x10')))))) ||
             (((schKX0[0] == '\x12' || (schKX0[0] == '\x0e')) || (schKX0[0] == '\x02')))) {
            bByte = '\x17';
            goto LAB_00075414;
          }
        }
LAB_00075530:
        BVar3 = 0;
        goto LAB_00075426;
      }
LAB_0007540c:
      bByte = '\f';
    }
  }
  else if (schKX0[2] == '\a') {
    if (wWan == 0xb2eb) {
      if (wCX1 == 0xae68) {
LAB_000759a5:
        bByte = '\x19';
      }
      else {
        YongYonAdd(pbArray,pnSize,'\v');
        bByte = '\x19';
      }
    }
    else if ((wWan == 0xbb3b) || (wWan == 0xac77)) {
      YongYonAdd(pbArray,pnSize,'\f');
      bByte = '\x1a';
    }
    else {
      if (((((((wWan == 0xb3cb) || (wWan == 0xad73)) || (wWan == 0xbc8b)) ||
            ((wWan == 0xb51b || (wWan == 0xbbff)))) ||
           ((wWan == 0xb72f || ((wWan == 0xbed7 || (wWan == 0xbc1b)))))) || (wWan == 0xc5bb)) ||
         (wWan == 0xc3df)) goto LAB_000753f7;
      if ((((wWan != 0xae37) && (wWan != 0xceeb)) && (wWan != 0xb20b)) &&
         (((wWan != 0xacaf && (wWan != 0xbd87)) && ((wWan != 0xb4e3 && (wWan != 0xc2e3))))))
      goto LAB_00075530;
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 == 1) goto LAB_000759a5;
      bByte = '\x1a';
    }
  }
  else if (schKX0[2] == '\x13') {
    if ((((wWan == 0xbc27) || (wWan == 0xae43)) ||
        (((wWan == 0xbe57 ||
          ((((wWan == 0xbc97 || (wWan == 0xc2ef)) || (wWan == 0xc19f)) ||
           ((wWan == 0xcacf || (wWan == 0xc53b)))))) || (wWan == 0xb057)))) ||
       (((wWan == 0xaf3f || (wWan == 0xc557)) || ((wWan == 0xbe8f || (wWan == 0xc6c3)))))) {
LAB_000753f7:
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 != 1) goto LAB_0007540c;
LAB_000757a0:
      bByte = '\v';
    }
    else {
      if ((((wWan != 0xb0ab) && (wWan != 0xae0b)) && (wWan != 0xbb47)) &&
         (((wWan != 0xbd93 && (wWan != 0xc90f)) &&
          ((wWan != 0xc22b &&
           (((wWan != 0xc9d3 && (wWan != 0xc813)) &&
            ((wWan != 0xc787 && ((wWan != 0xc7a3 && (wWan != 0xbabb)))))))))))) goto LAB_00075530;
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 == 1) {
        bByte = '\x1b';
      }
      else {
        bByte = '\x1c';
      }
    }
  }
  else if (schKX0[2] == '\b') {
    iVar4 = CheckChosong(schKX0[1]);
    if (iVar4 == 1) {
      bByte = '\x1d';
    }
    else {
      bByte = '\x1e';
    }
  }
  else {
    if (schKX0[2] == '\x14') goto LAB_0007540c;
    if (((schKX0[2] != '\x06') || (schKX0[1] != '\x01')) ||
       ((schKX0[0] != '\a' && (schKX0[0] != '\f')))) {
      if (wWan == 0xbc49) goto LAB_000757a0;
      goto LAB_000753f7;
    }
    bByte = '\r';
  }
LAB_00075414:
  BVar3 = YongYonAdd(pbArray,pnSize,bByte);
LAB_00075426:
  if (iVar1 != *(int *)(in_GS_OFFSET + 0x14)) {
    __stack_chk_fail_local();
  }
  return BVar3;
}

