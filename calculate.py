RightAscensionHours = 3
RightAscensionMinutes = 24
RightAscensionSeconds = 29

DeclinationDegrees = 49
DeclinationMinutes = 51
DeclinationSeconds = 39

print(f"\nRightAscension = {((RightAscensionHours * 15) + (RightAscensionMinutes/4) + (RightAscensionSeconds/240))/1}\n")
print(f"Declination = {DeclinationDegrees + (DeclinationMinutes/60) + (DeclinationSeconds/3600)}\n")