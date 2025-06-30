if !exist("dat1") || !exist("dat2") || !exist("dat3")
  dat1=load("IASI_SND_02_M01_20200909010854Z_20200909025054Z_N_O_20200909020541Z.nc");
  dat2=load("L2_coda_20200909.nc");
  dat3=load("IASISND02_20200909010854Z_20200909025054Z_epct_74ffd501_F.nc");
end

iline=1;

lat1=dat1.lat(:);
lat2=dat2.EARTH_LOCATION(1,:,:)(:);
lat3=dat3.lat(:)*0.0001;

lon1=dat1.lon(:);
lon2=dat2.EARTH_LOCATION(2,:,:)(:);
lon3=dat3.lon(:)*0.0001;

clf;
subplot(2,1,1);
hist([lat1,lat2,lat3],30);
legend(["iasinat";"CODA";"EUMETSAT"]);
title("Latitude / EARTH\\_LOCATION 1");

subplot(2,1,2);
hist([lon1,lon2,lon3],30);
legend(["iasinat";"CODA";"EUMETSAT"]);
title("Longitude / EARTH\\_LOCATION 2");
