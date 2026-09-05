function rallyIsTypingTarget(target)
{
    if (!target || !target.tagName)
    {
        return false;
    }

    const tag = target.tagName.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select')
    {
        return true;
    }

    return !!target.isContentEditable;
}


function rallyBindKeyboardControls()
{
    if (window.__rallyKeyboardBound)
    {
        return;
    }
    window.__rallyKeyboardBound = true;

    document.addEventListener('keydown', function(event) {
        if (!rallyHasAnimationData())
        {
            return;
        }
        if (rallyIsTypingTarget(event.target))
        {
            return;
        }
        if (event.altKey || event.ctrlKey || event.metaKey)
        {
            return;
        }

        const key = event.key;
        if (key === ' ' || key === 'Spacebar')
        {
            // Let focused control buttons keep native Space activation, but treat
            // the seek slider as play/pause (keyboard click coords are unreliable).
            const onControl = event.target && event.target.closest
                && event.target.closest('button, a, [role="button"]');
            const onSeekSlider = event.target && event.target.id === 'rallyProgressTrack';
            if (onControl && !onSeekSlider)
            {
                return;
            }
            event.preventDefault();
            rallyTogglePlayPause();
            return;
        }

        if (key === 'ArrowLeft')
        {
            event.preventDefault();
            if (event.shiftKey || myRallyPlayer.playing)
            {
                rallySeekByTurns(-1);
            }
            else
            {
                rallySeekByCpuSteps(-1);
            }
            return;
        }

        if (key === 'ArrowRight')
        {
            event.preventDefault();
            if (event.shiftKey || myRallyPlayer.playing)
            {
                rallySeekByTurns(1);
            }
            else
            {
                rallySeekByCpuSteps(1);
            }
            return;
        }

        if (key === 'Home')
        {
            event.preventDefault();
            rallySeekToRatio(0);
            return;
        }

        if (key === 'End')
        {
            event.preventDefault();
            rallySeekToRatio(1);
        }
    });
}


function rallyBindTransportControls()
{
    if (window.__rallyTransportBound)
    {
        return;
    }
    window.__rallyTransportBound = true;

    const playPause = document.getElementById('rallyPlayPause');
    if (playPause)
    {
        playPause.addEventListener('click', function() {
            rallyTogglePlayPause();
        });
    }

    const restart = document.getElementById('rallyRestart');
    if (restart)
    {
        restart.addEventListener('click', rallyRestart);
    }

    const speedButtons = document.querySelectorAll('.rally-view-speed-button');
    for (let i = 0; i < speedButtons.length; i++)
    {
        speedButtons[i].addEventListener('click', function(event) {
            const speed = Number(event.currentTarget.getAttribute('data-speed'));
            if (speed > 0)
            {
                rallySetSpeed(speed);
            }
        });
    }

    const track = document.getElementById('rallyProgressTrack');
    if (track)
    {
        track.addEventListener('click', function(event) {
            // Keyboard-activated clicks have detail 0 and unusable coordinates.
            if (event.detail === 0)
            {
                return;
            }
            const rect = track.getBoundingClientRect();
            if (rect.width <= 0)
            {
                return;
            }
            rallySeekToRatio((event.clientX - rect.left) / rect.width);
        });
    }

    rallyBindKeyboardControls();
}
