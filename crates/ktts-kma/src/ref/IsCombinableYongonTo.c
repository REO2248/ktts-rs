

BOOLT IsCombinableYongonTo(char chNewTag,BOOLT bNew,LPSMORPHCANDINFO psCand,char *pchPyogiStr)

{
  char cVar1;
  BYTE BVar2;
  int iVar3;
  uint uVar4;
  BOOLT BVar5;
  BOOLT BVar6;
  char *pcVar7;
  int iVar8;
  char *pcVar9;
  int in_GS_OFFSET;
  bool bVar10;
  byte bVar11;
  char local_180;
  BOOLT fClose;
  WCHART swKr [100];
  int local_8c;
  BYTE sbYNumArray [2];
  char schCompStr [100];
  
  pcVar7 = PTR_gschMorphCandStr_00095164;
  bVar11 = 0;
  iVar3 = *(int *)(in_GS_OFFSET + 0x14);
  if (bNew != 0) goto _L40;
  cVar1 = PTR_gschMorphCandStr_00095164[*(int *)PTR_gnMorphCandStr_000952f8];
  BVar5 = IsClosedSyllableKRoot(PTR_gschMorphCandStr_00095164);
  uVar4 = psCand->unToInfo;
  if ((uVar4 & 0x100) == 0) {
    if ((((*pcVar7 != DAT_0008a3d6) || (pcVar7[1] != DAT_0008a3d7)) || (pcVar7[2] != DAT_0008a3d8))
       || (pcVar7[3] != DAT_0008a3d9)) {
      if (((*pcVar7 == DAT_0008a3da) && (pcVar7[1] == DAT_0008a3db)) && (pcVar7[2] == DAT_0008a3dc))
      {
        uVar4 = 0;
        goto LAB_00075d25;
      }
      goto LAB_00075df8;
    }
LAB_00075dd0:
    uVar4 = 0;
    goto LAB_00075d25;
  }
LAB_00075df8:
  local_180 = (char)uVar4;
  if ((((byte)(chNewTag + 0xbdU) < 2) &&
      ((((*pcVar7 != DAT_0008a3d6 || (pcVar7[1] != DAT_0008a3d7)) ||
        ((pcVar7[2] != DAT_0008a3d8 || (pcVar7[3] != DAT_0008a3d9)))) && (-1 < local_180)))) ||
     ((((chNewTag == 'B' || (chNewTag == '@')) && (cVar1 != 'H')) && ((uVar4 & 0x40) == 0))))
  goto LAB_00075dd0;
  ConvPyogiToUniWan(pcVar7,swKr);
  Wcsadd(swKr,0xb2e4);
  local_8c = 0;
  BVar6 = GetYongYonNum(swKr,sbYNumArray,&local_8c);
  if (BVar6 == 0) goto _L40;
  BVar2 = sbYNumArray[0];
  if ((chNewTag != '@') && ((chNewTag != 'C' || (local_8c != 1)))) {
    BVar2 = sbYNumArray[1];
  }
  uVar4 = psCand->unToInfo;
  if ((uVar4 & 1) != 0) goto _L40;
  if ((uVar4 & 2) != 0) {
    BVar6 = IsBadTimTo(psCand->chPumsa,psCand->schMorphStr);
    if (BVar6 != 0) {
      if ((BVar5 != 0) && (cVar1 != 'L')) {
        if (((byte)(BVar2 - 0x34) < 8) &&
           (((*pcVar7 == DAT_0008a40c && (pcVar7[1] == DAT_0008a40d)) && (pcVar7[2] == DAT_0008a40e)
            ))) {
          uVar4 = (uint)(pcVar7[3] == DAT_0008a40f);
          goto LAB_00075d25;
        }
        goto LAB_00075dd0;
      }
      goto _L40;
    }
    iVar8 = StrcmpLeft(psCand->schMorphStr,"n_Nda");
    if ((iVar8 == 0) || (psCand->schMorphStr[0] == 's')) {
      uVar4 = (uint)(BVar5 != 0);
      goto LAB_00075d25;
    }
    uVar4 = psCand->unToInfo;
  }
  if ((uVar4 & 4) == 0) {
    if ((uVar4 & 8) == 0) {
      if ((uVar4 & 0x10) != 0) {
        if (BVar5 == 0) {
LAB_000761fc:
          uVar4 = (uint)(psCand->schMorphStr[0] != '_');
          goto LAB_00075d25;
        }
        if (psCand->schMorphStr[0] != '_') {
          pcVar7 = "LBH";
LAB_00075f10:
          pcVar7 = strchr(pcVar7,(int)cVar1);
          uVar4 = (uint)(pcVar7 != (char *)0x0);
          goto LAB_00075d25;
        }
      }
    }
    else {
      __sprintf_chk(schCompStr,1,100,&DAT_000854a6,pcVar7,psCand->schMorphStr);
      if ((((byte)(BVar2 - 0x1b) < 2) || (BVar2 == '7')) &&
         ((psCand->schMorphStr[0] == DAT_00088f76 &&
          ((psCand->schMorphStr[1] == DAT_00088f77 && (psCand->schMorphStr[2] == DAT_00088f78))))))
      {
        pcVar7 = strstr(pchPyogiStr,schCompStr);
        uVar4 = (uint)(pcVar7 == (char *)0x0);
        goto LAB_00075d25;
      }
      if ((((byte)(BVar2 - 0x1d) < 2) || (BVar2 == '<')) || (BVar2 == '=')) {
        pcVar7 = strstr(pchPyogiStr,schCompStr);
        uVar4 = (uint)(pcVar7 == (char *)0x0);
        goto LAB_00075d25;
      }
      if (BVar5 == 0) goto LAB_000761fc;
      if (psCand->schMorphStr[0] != '_') {
        pcVar7 = "BH";
        goto LAB_00075f10;
      }
    }
    goto _L40;
  }
  switch(BVar2) {
  case '\x01':
  case '\t':
  case '\n':
  case '\r':
  case '\x0e':
  case '\x10':
  case '\x16':
  case '\x19':
  case '\x1b':
  case '\x1d':
  case '$':
  case '*':
  case ',':
  case '-':
  case '/':
  case '4':
  case '7':
  case '8':
  case '9':
  case ':':
  case ';':
  case '<':
    if (psCand->schMorphStr[0] == 'e') goto LAB_00075dd0;
    goto _L63;
  case '\x02':
    if ((psCand->schMorphStr[0] == 'a') ||
       ((psCand->schMorphStr[0] == s_Sentence_0008575e[7] &&
        (psCand->schMorphStr[1] == s_Sentence_0008575e[8])))) {
      bVar10 = true;
      iVar8 = 6;
      pcVar9 = "maJse";
      do {
        if (iVar8 == 0) break;
        iVar8 = iVar8 + -1;
        bVar10 = *pcVar7 == *pcVar9;
        pcVar7 = pcVar7 + (uint)bVar11 * -2 + 1;
        pcVar9 = pcVar9 + (uint)bVar11 * -2 + 1;
      } while (bVar10);
      uVar4 = (uint)bVar10;
      goto LAB_00075d25;
    }
    break;
  case '\x03':
  case '\x06':
  case '\x14':
  case '!':
  case '2':
    uVar4 = (uint)(psCand->schMorphStr[0] != 'a');
    goto LAB_00075d25;
  case '\x04':
  case '\a':
  case '\b':
  case '\x0f':
    if (psCand->schMorphStr[0] == 'a') goto LAB_00075dd0;
    if (psCand->schMorphStr[0] == DAT_00089a93) {
      uVar4 = (uint)(psCand->schMorphStr[1] != DAT_00089a94);
      goto LAB_00075d25;
    }
    break;
  case '\x05':
  case '\v':
  case '5':
    goto _L63;
  case '\f':
  case '\x11':
  case '\x12':
  case '\x13':
  case '\x17':
  case '\x18':
  case '\x1a':
  case '\x1c':
  case '\x1e':
  case '\x1f':
  case ' ':
  case '%':
  case '+':
  case '.':
  case '0':
  case '1':
  case '6':
  case '=':
    if (psCand->schMorphStr[0] == 'a') goto LAB_00075dd0;
    goto _L63;
  case '#':
  case '&':
  case '(':
    uVar4 = (uint)(psCand->schMorphStr[0] != 'e' && psCand->schMorphStr[0] != 'a');
    goto LAB_00075d25;
  }
_L40:
  uVar4 = 1;
LAB_00075d25:
  if (iVar3 == *(int *)(in_GS_OFFSET + 0x14)) {
    return uVar4;
  }
  __stack_chk_fail_local();
_L63:
  iVar8 = StrcmpLeft(psCand->schMorphStr,"ye");
  uVar4 = (uint)(iVar8 != 0);
  goto LAB_00075d25;
}

