var map = L.map('map').setView([37.75343840, -122.4913135], 13);

var osm = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
});

var mapbox = L.tileLayer('https://api.mapbox.com/styles/v1/{id}/tiles/{z}/{x}/{y}?access_token={accessToken}', {
    attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors, Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
    maxZoom: 18,
    id: 'mapbox/streets-v11',
    tileSize: 512,
		zoomOffset: -1,
		// TODO: remove this token
    accessToken: 'pk.eyJ1IjoicGV0ZXItcG9wb3YiLCJhIjoiY2tnODY4eW54MGR1OTMzb2N5ZDZpaWp2eCJ9.JjXtXQbFbaAJJJP5ev0LJw'
}).addTo(map);

var baseMaps = {
	"OSM": osm,
	"Mapbox streets": mapbox
};

L.control.layers(baseMaps, {}, {position: 'topleft'}).addTo(map);


var control = L.Routing.control(L.extend(window.lrmConfig, {
	waypoints: [
		L.latLng(37.75343840, -122.4913135),
		L.latLng(37.74982854, -122.4084312)
	],
	geocoder: L.Control.Geocoder.nominatim(),
	routeWhileDragging: true,
	reverseWaypoints: true,
	showAlternatives: true,
	lineOptions: {
		styles: [
			{color: 'blue', opacity: 0.5, weight: 5}
		]
	}
})).addTo(map);

L.Routing.errorControl(control).addTo(map);