import matplotlib.pyplot as plt
import numpy as np
from scipy.io import wavfile
from matplotlib.widgets import TextBox, Button

def plot_waveform_comparison(wav_file1, wav_file2):
    # 读取第一个wav文件
    sample_rate1, data1 = wavfile.read(wav_file1)
    # 读取第二个wav文件
    sample_rate2, data2 = wavfile.read(wav_file2)
    
    # 确保两个音频文件的采样率相同
    if sample_rate1 != sample_rate2:
        raise ValueError("Sample rates of the two audio files do not match.")
    
    # 计算帧数轴，每帧包含1152个数据点
    frame_size = 1152
    num_frames1 = len(data1) // frame_size
    num_frames2 = len(data2) // frame_size
    num_frames = min(num_frames1, num_frames2)
    frames = np.arange(num_frames)
    
    # 计算每帧的平均值作为波形数据
    frame_data1 = np.mean(data1[:num_frames * frame_size].reshape(-1, frame_size), axis=1)
    frame_data2 = np.mean(data2[:num_frames * frame_size].reshape(-1, frame_size), axis=1)
    
    # 向右平移
    shift_frames = 0
    shifted_data2 = np.roll(frame_data2, shift_frames)
    
    # 确保两个音频数据长度一致
    min_length = min(len(frame_data1), len(shifted_data2))
    frame_data1 = frame_data1[:min_length]
    shifted_data2 = shifted_data2[:min_length]
    frames = frames[:min_length]
    
    # 绘制波形图
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 6))  # 这里修改了图形大小
    ax1.plot(frames, frame_data1, label=wav_file1)
    ax1.plot(frames, shifted_data2, label=wav_file2)
    ax1.set_title('Waveform Comparison')
    ax1.set_ylabel('Amplitude')
    ax1.set_xlabel('Frame')
    ax1.set_xlim(0, frames[-1])
    ax1.legend()

    # 为输入框和按钮腾出空间
    plt.subplots_adjust(bottom=0.2)

    return fig, ax1, ax2, data1, data2, frame_size, num_frames

def compare_single_frame(ax, data1, data2, frame_size, frame_number):
    ax.clear()
    start_index = frame_number * frame_size
    end_index = start_index + frame_size

    frame1 = data1[start_index:end_index]
    frame2 = data2[start_index:end_index]

    ax.plot(range(frame_size), frame1, label='Audio 1')
    ax.plot(range(frame_size), frame2, label='Audio 2')
    ax.set_title(f'Frame {frame_number} Comparison')
    ax.set_xlabel('Sample')
    ax.set_ylabel('Amplitude')
    ax.legend()
    plt.draw()

# 使用示例
fig, ax1, ax2, data1, data2, frame_size, num_frames = plot_waveform_comparison('out.wav', '22.wav')

current_frame = 0

# 创建输入框
ax_textbox = plt.axes([0.47, 0.1, 0.06, 0.04])
text_box = TextBox(ax_textbox, '', initial=str(current_frame))

# 创建左右按钮
ax_left = plt.axes([0.45, 0.1, 0.02, 0.04])
btn_left = Button(ax_left, '<')

ax_right = plt.axes([0.53, 0.1, 0.02, 0.04])
btn_right = Button(ax_right, '>')

# 更新函数
def update_frame(frame):
    global current_frame
    current_frame = int(frame)
    current_frame = max(0, min(current_frame, num_frames - 1))
    compare_single_frame(ax2, data1, data2, frame_size, current_frame)
    text_box.set_val(str(current_frame))

# 输入框回调函数
def submit(text):
    update_frame(int(text))

# 按钮回调函数
def on_left_click(event):
    update_frame(current_frame - 1)

def on_right_click(event):
    update_frame(current_frame + 1)

# 实现直接缩放功能
def zoom_factory(ax, base_scale = 2.):
    def zoom_fun(event):
        # 确保事件发生在当前轴上
        if event.inaxes != ax:
            return

        # 获取当前x和y轴的限制
        cur_xlim = ax.get_xlim()
        cur_ylim = ax.get_ylim()

        xdata = event.xdata # 获取鼠标点击的x坐标
        ydata = event.ydata # 获取鼠标点击的y坐标

        if event.button == 'up':
            # 向上滚动，放大
            scale_factor = 1 / base_scale
        elif event.button == 'down':
            # 向下滚动，缩小
            scale_factor = base_scale
        else:
            # 其他情况，不进行缩放
            scale_factor = 1

        # 设置新的轴限制
        new_width = (cur_xlim[1] - cur_xlim[0]) * scale_factor
        new_height = (cur_ylim[1] - cur_ylim[0]) * scale_factor

        relx = (cur_xlim[1] - xdata)/(cur_xlim[1] - cur_xlim[0])
        rely = (cur_ylim[1] - ydata)/(cur_ylim[1] - cur_ylim[0])

        ax.set_xlim([xdata - new_width * (1-relx), xdata + new_width * (relx)])
        ax.set_ylim([ydata - new_height * (1-rely), ydata + new_height * (rely)])
        ax.figure.canvas.draw()

    fig = ax.get_figure() # 获取图形
    # 将滚轮事件与缩放函数连接
    fig.canvas.mpl_connect('scroll_event', zoom_fun)

    return zoom_fun

text_box.on_submit(submit)
btn_left.on_clicked(on_left_click)
btn_right.on_clicked(on_right_click)

# 初始化第一帧的比较
compare_single_frame(ax2, data1, data2, frame_size, current_frame)

# 为两个子图都添加缩放功能
zoom_factory(ax1)
zoom_factory(ax2)

# 默认选中平移工具
plt.gcf().canvas.toolbar.pan()

plt.tight_layout()
plt.show()
