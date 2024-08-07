import streamlit as st
import requests
import pandas as pd
import os
from streamlit_autorefresh import st_autorefresh
import plotly.express as px

st.set_page_config(layout="wide")

st.title('RustBalancer™')

st_autorefresh(interval=2000, key="datarefresh")

api_url = os.getenv('DEPLOYMENT_URL')

response = requests.get(api_url)
data = response.json()

df = pd.DataFrame(data)
st.write("### Data Overview")
st.dataframe(df)

st.write("### CPU Usage")
fig_cpu = px.bar(df, x='name', y='cpu_usage', title='CPU Usage')
st.plotly_chart(fig_cpu)

cpu_usage_pie = px.pie(df, names='name', values='cpu_usage', title='CPU Usage Distribution')
st.plotly_chart(cpu_usage_pie)

st.write("### Memory Usage")
fig_memory = px.bar(df, x='name', y='memory_usage', title='Memory Usage')
st.plotly_chart(fig_memory)

st.write("### Metrics")
col1, col2 = st.columns(2)
col1.metric("Average CPU Usage", f"{df['cpu_usage'].mean():.4f}")
col2.metric("Average Memory Usage", f"{df['memory_usage'].mean():.4f}")

for index, row in df.iterrows():
    with st.expander(f"Details for {row['name']}"):
        st.write(f"ID: {row['id']}")
        st.write(f"Image: {row['image']}")
        st.write(f"State: {row['state']}")
        st.write(f"Ports: {row['ports']}")
        st.write(f"Uptime: {row['uptime']}")
