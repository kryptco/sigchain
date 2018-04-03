chart = new Chartist.Line('.logs-chart', {
      labels: ['30m', '25m', '20m', '15m', '10m', '5m', '0m'],
      // Naming the series with the series object array notation
      series: [{
        name: 'series-1',
        data: [0, 0, 0, 0, 0, 0, 0],
      }]
    }, {

      fullWidth: true,
      height: '250px',
      axisY: {
        onlyInteger: true,
        offset: 20
      },
      series: {
        'series-1': {
          lineSmooth: Chartist.Interpolation.step(),
          showPoint: false,
          showArea: true,
          lineColor: "#5bc894",

        },
      }
    }, [
      ]);

chart.on('draw', function(data) {
  if(data.type === 'area') {
    // If the drawn element is a line we do a simple opacity fade in. This could also be achieved using CSS3 animations.
    data.element.animate({
      opacity: {
        // The delay when we like to start the animation
        begin: 1000,
        // Duration of the animation
        dur: 2000,
        // The value where the animation should start
        from: 0,
        // The value where it should end
        to: 1
      }
    });
  }
});

chart.on('created', function(data) {
   var path = document.getElementsByClassName('ct-line')[0];

   if (path) {
      var length = path.getTotalLength();

      // modify the dash offset
      var el = document.getElementsByClassName('ct-series-a')[0];

      if (el) {
        el.style["stroke-dasharray"] = length;
        el.style["stroke-dashoffset"] = length;
        console.log(length);
      }
   }
});