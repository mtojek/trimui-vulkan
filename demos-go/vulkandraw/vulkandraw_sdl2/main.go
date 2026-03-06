package main

import (
	"fmt"
	"log"
	"runtime"
	"time"

	"github.com/veandco/go-sdl2/sdl"
	"github.com/vulkan-go/demos/vulkandraw"
	vk "github.com/vulkan-go/vulkan"
	"github.com/xlab/closer"
)

var appInfo = &vk.ApplicationInfo{
	SType:              vk.StructureTypeApplicationInfo,
	ApiVersion:         vk.MakeVersion(1, 0, 0),
	ApplicationVersion: vk.MakeVersion(1, 0, 0),
	PApplicationName:   "VulkanDraw\x00",
	PEngineName:        "vulkango.com\x00",
}

func init() {
	runtime.LockOSThread()
	log.SetFlags(log.Lshortfile)
}

func main() {
	orPanic(sdl.Init(sdl.INIT_VIDEO | sdl.INIT_EVENTS))
	defer sdl.Quit()

	orPanic(sdl.VulkanLoadLibrary(""))
	defer sdl.VulkanUnloadLibrary()

	vk.SetGetInstanceProcAddr(sdl.VulkanGetVkGetInstanceProcAddr())
	orPanic(vk.Init())
	defer closer.Close()

	var (
		v   vulkandraw.VulkanDeviceInfo
		s   vulkandraw.VulkanSwapchainInfo
		r   vulkandraw.VulkanRenderInfo
		b   vulkandraw.VulkanBufferInfo
		gfx vulkandraw.VulkanGfxPipelineInfo
	)

	window, err := sdl.CreateWindow("VulkanDraw (SDL2)",
		sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		640, 480,
		sdl.WINDOW_VULKAN)
	orPanic(err)
	defer window.Destroy()

	createSurface := func(instance interface{}) uintptr {
		surfPtr, err := window.VulkanCreateSurface(instance)
		orPanic(err)
		return uintptr(surfPtr)
	}

	v, err = vulkandraw.NewVulkanDevice(appInfo,
		0,
		window.VulkanGetInstanceExtensions(),
		createSurface)
	orPanic(err)
	s, err = v.CreateSwapchain()
	orPanic(err)
	r, err = vulkandraw.CreateRenderer(v.Device, s.DisplayFormat, v.QueueFamilyIndex)
	orPanic(err)
	err = s.CreateFramebuffers(r.RenderPass, nil)
	orPanic(err)
	b, err = v.CreateBuffers()
	orPanic(err)
	gfx, err = vulkandraw.CreateGraphicsPipeline(v.Device, s.DisplaySize, r.RenderPass)
	orPanic(err)
	log.Println("[INFO] swapchain lengths:", s.SwapchainLen)
	err = r.CreateCommandBuffers(s.DefaultSwapchainLen())
	orPanic(err)

	doneC := make(chan struct{}, 2)
	exitC := make(chan struct{}, 2)
	defer closer.Bind(func() {
		exitC <- struct{}{}
		<-doneC
		log.Println("Bye!")
	})
	vulkandraw.VulkanInit(&v, &s, &r, &b, &gfx)

	fpsTicker := time.NewTicker(time.Second / 30)
	start := time.Now()
	frames := 0
_MainLoop:
	for {
		select {
		case <-exitC:
			fmt.Printf("FPS: %.2f\n", float64(frames)/time.Now().Sub(start).Seconds())
			vulkandraw.DestroyInOrder(&v, &s, &r, &b, &gfx)
			fpsTicker.Stop()
			doneC <- struct{}{}
			return
		case <-fpsTicker.C:
			frames++
			var event sdl.Event
			for event = sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
				switch t := event.(type) {
				case *sdl.KeyboardEvent:
					if t.Keysym.Sym == sdl.K_ESCAPE {
						exitC <- struct{}{}
						continue _MainLoop
					}
				case *sdl.QuitEvent:
					exitC <- struct{}{}
					continue _MainLoop
				}
			}
			if ok := vulkandraw.VulkanDrawFrame(v, s, r); !ok {
				log.Println("[WARN] draw skipped")
			}
		}
	}
}

func orPanic(err interface{}) {
	switch v := err.(type) {
	case error:
		if v != nil {
			panic(err)
		}
	case vk.Result:
		if err := vk.Error(v); err != nil {
			panic(err)
		}
	case bool:
		if !v {
			panic("condition failed: != true")
		}
	}
}
