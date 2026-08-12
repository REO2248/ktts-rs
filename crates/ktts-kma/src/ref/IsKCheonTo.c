

BOOLT IsKCheonTo(BYTE bPumsa)

{
  BOOLT BVar1;
  
  BVar1 = IsCaseTo(bPumsa);
  if (((BVar1 == 0) && (BVar1 = IsHelpTo(bPumsa), BVar1 == 0)) &&
     (BVar1 = IsPluralTo(bPumsa), BVar1 == 0)) {
    BVar1 = IsBagumYi(bPumsa);
    return (uint)(BVar1 != 0);
  }
  return 1;
}

