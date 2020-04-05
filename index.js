const Request = require('request');
const SpotifyAccessory = require('./homebridge_rusty_spotify.js').SpotifyAccessory;

function partial(fn /*, rest args */){
  return fn.bind.apply(fn, Array.apply(null, arguments));
}

module.exports = function(homebridge) {
  console.log("homebridge API version: " + homebridge.version);

  // Accessory must be created from PlatformAccessory Constructor
  Accessory = homebridge.platformAccessory;

  // Service and Characteristic are from hap-nodejs
  Service = homebridge.hap.Service;
  let Switch = new homebridge.hap.Service.Switch("SpotifyAccessory");
  Characteristic = homebridge.hap.Characteristic;

  // For platform plugin to be considered as dynamic platform plugin,
  // registerPlatform(pluginName, platformName, constructor, dynamic), dynamic must be true
  constructor = partial(SpotifyAccessory, Switch);
  homebridge.registerAccessory("homebridge-rusty-spotify", "SpotifyAccessory", constructor, true);
}
