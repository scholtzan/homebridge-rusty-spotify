const SpotifyAccessory = require('./homebridge_rusty_spotify.js').SpotifyAccessory;

function partial(fn /*, rest args */){
  return fn.bind.apply(fn, Array.apply(null, arguments));
}

module.exports = function(homebridge) {
  console.log("homebridge API version: " + homebridge.version);

  Accessory = homebridge.platformAccessory;
  Service = homebridge.hap.Service;
  let Switch = new Service.Lightbulb("Spotify");
  Characteristic = homebridge.hap.Characteristic;

  // we'll use brightness to control the volume
  // needs to be explicitly added, otherwise calling getCharacteristic() with a string won't work
  Switch.addCharacteristic(Characteristic.Brightness);

  constructor = partial(SpotifyAccessory, Switch);
  homebridge.registerAccessory("homebridge-rusty-spotify", "SpotifyAccessory", constructor, true);
}
