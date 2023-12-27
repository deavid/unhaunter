<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="A3x3x3" tilewidth="32" tileheight="32" tilecount="276" columns="12" objectalignment="bottom">
 <tileoffset x="-4" y="0"/>
 <grid orientation="isometric" width="24" height="12"/>
 <transformations hflip="1" vflip="0" rotate="0" preferuntransformed="0"/>
 <image source="../img/spritesheetA_3x3x3.png" width="384" height="704"/>
 <tile id="0" type="Floor"/>
 <tile id="1" type="Floor"/>
 <tile id="2" type="Floor"/>
 <tile id="4" type="Grass1:Floor:Std"/>
 <tile id="5" type="Grass2:Floor:Std"/>
 <tile id="41" type="WoodNW:Door:Closed">
  <properties>
   <property name="isOpen" type="bool" value="false"/>
  </properties>
 </tile>
 <tile id="42" type="WoodNE:Door:Closed">
  <properties>
   <property name="is_open" type="bool" value="false"/>
   <property name="onactivate_transform_to_id" type="int" value="44"/>
  </properties>
 </tile>
 <tile id="43" type="WoodNW:Door:Open"/>
 <tile id="44" type="WoodNE:Door:Open"/>
 <wangsets>
  <wangset name="Unnamed Set" type="corner" tile="-1">
   <wangcolor name="stones" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="grass" color="#00ff00" tile="-1" probability="1"/>
   <wangcolor name="asphalt" color="#0000ff" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="2" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="3" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="4" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="5" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="6" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="7" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="8" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="9" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="10" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="11" wangid="0,3,0,3,0,3,0,3"/>
  </wangset>
  <wangset name="Unnamed Set" type="edge" tile="-1">
   <wangcolor name="" color="#ff0000" tile="-1" probability="1"/>
  </wangset>
 </wangsets>
</tileset>
