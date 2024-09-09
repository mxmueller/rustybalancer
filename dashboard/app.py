import streamlit as st
import requests
import pandas as pd
import os
from streamlit_autorefresh import st_autorefresh
import plotly.express as px

st.set_page_config(layout="wide")

st.title('RustBalancer™')

st_autorefresh(interval=10000, key="datarefresh")

api_url = os.getenv('DEPLOYMENT_URL')

response = requests.get(api_url)
data = response.json()

df = pd.DataFrame(data)


def color_utilization_category(val):
    color = ''
    if val == 'LU':
        color = 'green'
    elif val == 'MU':
        color = 'orange'
    elif val == 'HU':
        color = 'red'
    elif val == 'SUNDOWN':
        color = 'purple'
    elif val == 'INIT':
        color = 'gray'
    return f'color: {color}'


styled_df = df.style.map(color_utilization_category, subset=['utilization_category'])

st.write("### Data Overview")
st.dataframe(styled_df)

col1, col2 = st.columns(2)

with col1:
    cpu_score_pie = px.pie(
        df,
        names='name',
        values='cpu_score',
        title='CPU Score Distribution',
        color_discrete_sequence=px.colors.sequential.Blues
    )
    st.plotly_chart(cpu_score_pie)

with col2:
    memory_score_pie = px.pie(
        df,
        names='name',
        values='memory_score',
        title='Memory Score Distribution',
        color_discrete_sequence=px.colors.sequential.Greens
    )
    st.plotly_chart(memory_score_pie)

# Weitere Spalten für die anderen Pie-Charts
col3, col4 = st.columns(2)

with col3:
    network_score_pie = px.pie(
        df,
        names='name',
        values='network_score',
        title='Network Score Distribution',
        color_discrete_sequence=px.colors.sequential.Purples
    )
    st.plotly_chart(network_score_pie)

with col4:
    availability_score_pie = px.pie(
        df,
        names='name',
        values='availability_score',
        title='Availability Score Distribution',
        color_discrete_sequence=px.colors.sequential.Oranges
    )
    st.plotly_chart(availability_score_pie)

st.write("### CPU Score")
fig_cpu = px.bar(
    df,
    x='name',
    y='cpu_score',
    title='CPU Score',
)
st.plotly_chart(fig_cpu)

st.write("### Memory Score")
fig_memory = px.bar(
    df,
    x='name',
    y='memory_score',
    title='Memory Score',
)
st.plotly_chart(fig_memory)

st.write("### Metrics")
col1, col2, col3 = st.columns(3)
col1.metric("Average CPU Score", f"{df['cpu_score'].mean():.4f}")
col2.metric("Average Memory Score", f"{df['memory_score'].mean():.4f}")
col3.metric("Average Overall Score", f"{df['overall_score'].mean():.4f}")

for index, row in df.iterrows():
    with st.expander(f"Details for {row['name']}"):
        st.write(f"ID: {row['id']}")
        st.write(f"CPU Score: {row['cpu_score']}")
        st.write(f"Memory Score: {row['memory_score']}")
        st.write(f"Network Score: {row['network_score']}")
        st.write(f"Availability Score: {row['availability_score']}")
        st.write(f"Overall Score: {row['overall_score']}")
        st.write(f"Utilization Category: {row['utilization_category']}")
