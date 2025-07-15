if !exist("dat1") || !exist("dat2")
  dat1=load("IASI_SND_02_M02_20210106104157Z_20210106122356Z_N_O_20210106122619Z.nc");
  dat2=load("L2_coda_20210106.nc");
end

iline=1;

lat1=dat1.lat(:);
lat2=dat2.EARTH_LOCATION(1,:,:)(:);

lon1=dat1.lon(:);
lon2=dat2.EARTH_LOCATION(2,:,:)(:);

st1=dat1.surface_temperature;
st2=dat2.SURFACE_TEMPERATURE;

sz1=dat1.surface_z;
sz2=dat2.SURFACE_Z;

global plot_nr;
global plot_nc;
global plot_iplot;

plot_nr=2;plot_nc=2;plot_iplot=0;

function next_plot()
  global plot_nr;
  global plot_nc;
  global plot_iplot;

  plot_iplot+=1;
  subplot(plot_nr,plot_nc,plot_iplot);
end

clf;

next_plot;
hist([lat1,lat2],30);
legend(["iasinat";"CODA"]);
title("Latitude / EARTH\\_LOCATION 1");

next_plot;
hist([lon1,lon2],30);
legend(["iasinat";"CODA"]);
title("Longitude / EARTH\\_LOCATION 2");

istgood1=find(!isnan(st1(:)));
istgood2=find(st2(:) < 655.0);

next_plot;
hist([st1(:)(istgood1),st2(:)(istgood2)],30);
legend(["iasinat";"CODA"]);
title("Surface temperature");

next_plot;
hist([sz1(:)(istgood1),sz2(:)(istgood2)],30);
legend(["iasinat";"CODA"]);
title("Elevation");
