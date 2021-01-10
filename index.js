const SpotifyPlatform = require('./homebridge_rusty_spotify.js').SpotifyPlatform;

function partial(fn /*, rest args */){
  return fn.bind.apply(fn, Array.apply(null, arguments));
}

module.exports = function(homebridge) {
  Accessory = homebridge.platformAccessory;
  Service = homebridge.hap.Service;
  UUIDGen = homebridge.hap.uuid;
  Characteristic = homebridge.hap.Characteristic;

  createLight = function(name) {
    let newSwitch = new Service.Lightbulb(name);
    // we'll use brightness to control the volume
    newSwitch.addCharacteristic(Characteristic.Brightness);
    return newSwitch;
  }

  createSpeaker = function(name) {
    let newSpeaker = new Service.Speaker(name);
    newSpeaker.addCharacteristic(Characteristic.Volume);
    return newSpeaker;
  }

  constructor = partial(SpotifyPlatform, homebridge);
  homebridge.registerPlatform("homebridge-rusty-spotify", "Spotify", constructor, true);
}
