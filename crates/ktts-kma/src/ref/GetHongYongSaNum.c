

BOOLT GetHongYongSaNum(WCHART *pwVstr,BYTE *pbArray,int *pnSize)

{
  int iVar1;
  size_t sVar2;
  BOOLT BVar3;
  int iVar4;
  WCHART wWan;
  int in_GS_OFFSET;
  WCHART wCX0;
  char schKX1 [4];
  char schKX0 [4];
  
  iVar1 = *(int *)(in_GS_OFFSET + 0x14);
  sVar2 = Wcslen(pwVstr);
  if ((int)sVar2 < 2) {
    wCX0 = 0;
    wWan = 0;
  }
  else {
    wCX0 = pwVstr[sVar2 - 2];
    if (sVar2 == 2) {
      wWan = 0;
    }
    else {
      wWan = pwVstr[sVar2 - 3];
    }
  }
  ConvUniCodeToCVC(wCX0,schKX0);
  ConvUniCodeToCVC(wWan,schKX1);
  if (schKX0[2] == '\0') {
    if (schKX0[1] == '\x01') {
      if (wCX0 == 0xd558) {
        BVar3 = YongYonAdd(pbArray,pnSize,'2');
      }
      else {
        BVar3 = YongYonAdd(pbArray,pnSize,'$');
      }
      goto LAB_00074dd0;
    }
    if ((schKX0[1] == '\x02') || (schKX0[1] == '\x06')) {
      BVar3 = YongYonAdd(pbArray,pnSize,'%');
      goto LAB_00074dd0;
    }
    if ((schKX0[1] == '\x11') || (schKX0[1] == '\x14')) {
      BVar3 = YongYonAdd(pbArray,pnSize,'&');
      goto LAB_00074dd0;
    }
    if (schKX0[1] == '\t') {
      BVar3 = YongYonAdd(pbArray,pnSize,'\'');
      goto LAB_00074dd0;
    }
    if (schKX0[1] == '\x12') {
      BVar3 = YongYonAdd(pbArray,pnSize,'(');
      goto LAB_00074dd0;
    }
    if (schKX0[1] == '\x15') {
      BVar3 = YongYonAdd(pbArray,pnSize,')');
      goto LAB_00074dd0;
    }
    if (schKX0[1] != '\x13') {
      if ((wWan == 0xc5b4) && (wCX0 == 0xca4c)) {
        BVar3 = YongYonAdd(pbArray,pnSize,'3');
        goto LAB_00074dd0;
      }
LAB_00074d16:
      BVar3 = 0;
      goto LAB_00074dd0;
    }
    iVar4 = CheckChosong(schKX1[1]);
    if (wCX0 == 0xb974) {
      if (((wWan == 0xb178) || (wWan == 0xd478)) || (wWan == 0xb204)) {
        BVar3 = YongYonAdd(pbArray,pnSize,'1');
        goto LAB_00074dd0;
      }
      if (iVar4 == 1) {
        if (wWan == 0) {
          BVar3 = YongYonAdd(pbArray,pnSize,'/');
          goto LAB_00074dd0;
        }
LAB_00075222:
        if (wWan != 0xc5b4) goto LAB_00074d16;
      }
      else if (iVar4 != 2) goto LAB_00075222;
      BVar3 = YongYonAdd(pbArray,pnSize,'0');
      goto LAB_00074dd0;
    }
    if (iVar4 != 1) {
      if (iVar4 != 2) goto LAB_00075134;
LAB_0007513c:
      BVar3 = YongYonAdd(pbArray,pnSize,'.');
      goto LAB_00074dd0;
    }
    if (wWan == 0) {
      BVar3 = YongYonAdd(pbArray,pnSize,'-');
      goto LAB_00074dd0;
    }
LAB_00075134:
    if (wWan == 0) goto LAB_0007513c;
LAB_00074ea6:
    BVar3 = 1;
    goto LAB_00074dd0;
  }
  if (schKX0[2] != '\x11') {
    if (wWan == 0xb0ab) {
      BVar3 = YongYonAdd(pbArray,pnSize,'7');
      goto LAB_00074dd0;
    }
    if (schKX0[2] == '\x1b') {
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 == 1) {
        if (schKX0[1] == '\x03') {
          BVar3 = YongYonAdd(pbArray,pnSize,'9');
        }
        else {
          BVar3 = YongYonAdd(pbArray,pnSize,'8');
        }
      }
      else if (schKX0[1] == '\a') {
        BVar3 = YongYonAdd(pbArray,pnSize,';');
      }
      else {
        BVar3 = YongYonAdd(pbArray,pnSize,':');
      }
      goto LAB_00074dd0;
    }
    if (schKX0[2] == '\b') {
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 == 1) {
        BVar3 = YongYonAdd(pbArray,pnSize,'<');
      }
      else {
        BVar3 = YongYonAdd(pbArray,pnSize,'=');
      }
      goto LAB_00074dd0;
    }
    if (((schKX0[2] == '\x06') && (schKX0[1] == '\x01')) &&
       ((schKX0[0] == '\a' || (schKX0[0] == '\f')))) {
      BVar3 = YongYonAdd(pbArray,pnSize,',');
      goto LAB_00074dd0;
    }
    iVar4 = CheckChosong(schKX0[1]);
    if (iVar4 != 1) goto LAB_00074d66;
LAB_00074f4d:
    BVar3 = YongYonAdd(pbArray,pnSize,'*');
    goto LAB_00074dd0;
  }
  if (wWan == 0xacf1) {
LAB_00075076:
    BVar3 = YongYonAdd(pbArray,pnSize,'4');
    goto LAB_00074dd0;
  }
  if (wWan == 0xc5b4) {
    if (wCX0 != 0xc90d) goto LAB_00074e16;
LAB_00074d66:
    BVar3 = YongYonAdd(pbArray,pnSize,'+');
  }
  else {
    if (wWan == 0xb2c8) {
      if (wCX0 == 0xaf3d) goto LAB_00075076;
LAB_00074e16:
      if (((schKX0[0] == '\t') || (schKX0[0] == '\f')) || (schKX0[0] == '\r')) {
        iVar4 = CheckChosong(schKX0[1]);
        if (iVar4 == 1) goto LAB_00074f4d;
        goto LAB_00074d66;
      }
      if (((((schKX0[0] != '\x03') && (schKX0[0] != '\x01')) && (schKX0[0] != '\x04')) &&
          ((schKX0[0] != '\x06' && (schKX0[0] != '\a')))) &&
         (((schKX0[0] != '\b' && ((schKX0[0] != '\n' && (schKX0[0] != '\x0f')))) &&
          ((schKX0[0] != '\x10' &&
           ((((schKX0[0] != '\x12' && (schKX0[0] != '\x0e')) && (schKX0[0] != '\x02')) &&
            ((schKX0[0] != '\x11' && (schKX0[0] != '\x13')))))))))) goto LAB_00074ea6;
      iVar4 = CheckChosong(schKX0[1]);
      if (iVar4 == 1) {
        BVar3 = YongYonAdd(pbArray,pnSize,'5');
        goto LAB_00074dd0;
      }
    }
    else if (wWan != 0xc5fd) goto LAB_00074e16;
    BVar3 = YongYonAdd(pbArray,pnSize,'6');
  }
LAB_00074dd0:
  if (iVar1 != *(int *)(in_GS_OFFSET + 0x14)) {
    __stack_chk_fail_local();
  }
  return BVar3;
}

