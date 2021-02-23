$(function(){
    var trigger = 0;
    $('#nav-btn').click(function () {
        if (trigger == 0) {
            $('.am-collapse').show();
            $('#collapse-head').fadeIn(500);
            $('#nav-backgroud-layer').show().animate({height:"1000px"});

            $('.am-nav li a').removeClass('animated fadeOutUp');
            $('.collapse-head').removeClass('animated fadeOutUp');


            $('.am-nav li a').addClass('animated fadeInDown');
            $('.collapse-head').addClass('animated fadeInDown');

            trigger = 1;
        }else {
            $('.am-nav li a').removeClass('animated fadeInDown');
            $('.collapse-head').removeClass('animated fadeInDown');

            $('.am-nav li a').addClass('animated fadeOutUp');
            $('.collapse-head').addClass('animated fadeOutUp');

            setTimeout(function () {
                $('.am-collapse').hide();
                $('#nav-backgroud-layer').css({height:"300px"}).hide();
            },500);

            trigger = 0;
        }



    })
})